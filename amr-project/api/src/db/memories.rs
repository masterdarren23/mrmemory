use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::*;

/// Row from the memories table.
#[derive(Debug, sqlx::FromRow)]
pub struct MemoryRow {
    pub id: Uuid,
    pub external_id: String,
    pub tenant_id: Uuid,
    pub agent_id: String,
    pub namespace: String,
    pub content: String,
    pub tags: Vec<String>,
    pub metadata: serde_json::Value,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl MemoryRow {
    pub fn to_response(&self) -> MemoryResponse {
        MemoryResponse {
            id: self.external_id.clone(),
            content: self.content.clone(),
            tags: self.tags.clone(),
            namespace: self.namespace.clone(),
            agent_id: self.agent_id.clone(),
            metadata: self.metadata.clone(),
            expires_at: self.expires_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

/// Insert a new memory. Returns the created row.
pub async fn insert_memory(
    db: &PgPool,
    tenant_id: Uuid,
    req: &CreateMemoryRequest,
) -> Result<MemoryRow, AppError> {
    let id = Uuid::new_v4();
    let external_id = format!(
        "mem_{}",
        id.simple().to_string().get(..12).unwrap_or("000000000000")
    );
    let agent_id = req.agent_id.clone().unwrap_or_else(|| "default".into());
    let now = Utc::now();
    let expires_at = req.ttl_seconds.map(|s| now + chrono::Duration::seconds(s));

    let row: MemoryRow = sqlx::query_as(
        "INSERT INTO memories (id, external_id, tenant_id, agent_id, namespace, content, tags, metadata, expires_at, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $10) RETURNING id, external_id, tenant_id, agent_id, namespace, content, tags, metadata, expires_at, created_at, updated_at"
    )
    .bind(id)
    .bind(&external_id)
    .bind(tenant_id)
    .bind(&agent_id)
    .bind(&req.namespace)
    .bind(&req.content)
    .bind(&req.tags)
    .bind(&req.metadata)
    .bind(expires_at)
    .bind(now)
    .fetch_one(db)
    .await?;

    Ok(row)
}

/// List memories with filters, pagination, time ordering.
pub async fn list_memories(
    db: &PgPool,
    tenant_id: Uuid,
    namespace: Option<&str>,
    agent_id: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<(Vec<MemoryRow>, i64), AppError> {
    let total: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM memories WHERE tenant_id = $1 AND ($2::TEXT IS NULL OR namespace = $2) AND ($3::TEXT IS NULL OR agent_id = $3)"
    )
    .bind(tenant_id)
    .bind(namespace)
    .bind(agent_id)
    .fetch_one(db)
    .await?;

    let rows: Vec<MemoryRow> = sqlx::query_as(
        "SELECT id, external_id, tenant_id, agent_id, namespace, content, tags, metadata, expires_at, created_at, updated_at FROM memories WHERE tenant_id = $1 AND ($2::TEXT IS NULL OR namespace = $2) AND ($3::TEXT IS NULL OR agent_id = $3) ORDER BY created_at DESC LIMIT $4 OFFSET $5"
    )
    .bind(tenant_id)
    .bind(namespace)
    .bind(agent_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(db)
    .await?;

    Ok((rows, total.0))
}

/// Get a single memory by external_id, scoped to tenant.
pub async fn get_memory_by_external_id(
    db: &PgPool,
    tenant_id: Uuid,
    external_id: &str,
) -> Result<Option<MemoryRow>, AppError> {
    let row: Option<MemoryRow> = sqlx::query_as(
        "SELECT id, external_id, tenant_id, agent_id, namespace, content, tags, metadata, expires_at, created_at, updated_at FROM memories WHERE tenant_id = $1 AND external_id = $2"
    )
    .bind(tenant_id)
    .bind(external_id)
    .fetch_optional(db)
    .await?;

    Ok(row)
}

/// Count memories for a tenant+namespace.
pub async fn count_memories_in_namespace(
    db: &PgPool,
    tenant_id: Uuid,
    namespace: &str,
) -> Result<i64, AppError> {
    let (count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM memories WHERE tenant_id = $1 AND namespace = $2")
            .bind(tenant_id)
            .bind(namespace)
            .fetch_one(db)
            .await?;

    Ok(count)
}

/// Delete expired memories. Returns the external_ids of pruned rows.
pub async fn prune_expired_memories(db: &PgPool) -> Result<Vec<String>, AppError> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "DELETE FROM memories WHERE expires_at IS NOT NULL AND expires_at < NOW() RETURNING external_id",
    )
    .fetch_all(db)
    .await?;

    Ok(rows.into_iter().map(|(eid,)| eid).collect())
}

/// Update a memory's content, tags, and/or metadata. Returns updated row.
pub async fn update_memory(
    db: &PgPool,
    tenant_id: Uuid,
    external_id: &str,
    content: Option<&str>,
    tags: Option<&[String]>,
    metadata: Option<&serde_json::Value>,
) -> Result<Option<MemoryRow>, AppError> {
    let now = Utc::now();
    let row: Option<MemoryRow> = sqlx::query_as(
        "UPDATE memories SET \
         content = COALESCE($3, content), \
         tags = COALESCE($4, tags), \
         metadata = COALESCE($5, metadata), \
         updated_at = $6 \
         WHERE tenant_id = $1 AND external_id = $2 \
         RETURNING id, external_id, tenant_id, agent_id, namespace, content, tags, metadata, expires_at, created_at, updated_at"
    )
    .bind(tenant_id)
    .bind(external_id)
    .bind(content)
    .bind(tags)
    .bind(metadata)
    .bind(now)
    .fetch_optional(db)
    .await?;

    Ok(row)
}

/// Bulk delete outdated memories by age and/or tags. Returns count deleted.
pub async fn delete_outdated(
    db: &PgPool,
    tenant_id: Uuid,
    older_than: Option<DateTime<Utc>>,
    tags: Option<&[String]>,
    namespace: Option<&str>,
    agent_id: Option<&str>,
    dry_run: bool,
) -> Result<usize, AppError> {
    if dry_run {
        let (count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM memories WHERE tenant_id = $1 \
             AND ($2::TIMESTAMPTZ IS NULL OR created_at < $2) \
             AND ($3::TEXT[] IS NULL OR tags @> $3) \
             AND ($4::TEXT IS NULL OR namespace = $4) \
             AND ($5::TEXT IS NULL OR agent_id = $5)"
        )
        .bind(tenant_id)
        .bind(older_than)
        .bind(tags)
        .bind(namespace)
        .bind(agent_id)
        .fetch_one(db)
        .await?;
        Ok(count as usize)
    } else {
        let result = sqlx::query(
            "DELETE FROM memories WHERE tenant_id = $1 \
             AND ($2::TIMESTAMPTZ IS NULL OR created_at < $2) \
             AND ($3::TEXT[] IS NULL OR tags @> $3) \
             AND ($4::TEXT IS NULL OR namespace = $4) \
             AND ($5::TEXT IS NULL OR agent_id = $5)"
        )
        .bind(tenant_id)
        .bind(older_than)
        .bind(tags)
        .bind(namespace)
        .bind(agent_id)
        .execute(db)
        .await?;
        Ok(result.rows_affected() as usize)
    }
}

/// Delete a memory by external_id. Returns true if deleted.
pub async fn delete_memory(
    db: &PgPool,
    tenant_id: Uuid,
    external_id: &str,
) -> Result<bool, AppError> {
    let result = sqlx::query("DELETE FROM memories WHERE tenant_id = $1 AND external_id = $2")
        .bind(tenant_id)
        .bind(external_id)
        .execute(db)
        .await?;

    Ok(result.rows_affected() > 0)
}
