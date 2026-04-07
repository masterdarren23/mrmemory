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
    pub is_compressed: bool,
    pub merged_from: Vec<String>,
    pub created_by: String,
    pub last_modified_by: Option<String>,
    pub confidence: Option<f32>,
    pub write_source: Option<String>,
    pub memory_type: String,
}

const MEMORY_COLUMNS: &str = "id, external_id, tenant_id, agent_id, namespace, content, tags, metadata, expires_at, created_at, updated_at, is_compressed, merged_from, created_by, last_modified_by, confidence, write_source, memory_type";

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
            is_compressed: self.is_compressed,
            merged_from: self.merged_from.clone(),
            created_by: Some(self.created_by.clone()),
            last_modified_by: self.last_modified_by.clone(),
            confidence: self.confidence,
            write_source: self.write_source.clone(),
            memory_type: self.memory_type.clone(),
        }
    }
}

/// Insert a new memory. Returns the created row.
pub async fn insert_memory(
    db: &PgPool,
    tenant_id: Uuid,
    req: &CreateMemoryRequest,
) -> Result<MemoryRow, AppError> {
    insert_memory_with_provenance(db, tenant_id, req, None, None, None, None).await
}

/// Insert a new memory with explicit provenance fields.
pub async fn insert_memory_with_provenance(
    db: &PgPool,
    tenant_id: Uuid,
    req: &CreateMemoryRequest,
    created_by_override: Option<&str>,
    confidence_override: Option<f32>,
    write_source_override: Option<&str>,
    memory_type_override: Option<&str>,
) -> Result<MemoryRow, AppError> {
    let id = Uuid::new_v4();
    let external_id = format!(
        "mem_{}",
        id.simple().to_string().get(..12).unwrap_or("000000000000")
    );
    let agent_id = req.agent_id.clone().unwrap_or_else(|| "default".into());
    let now = Utc::now();
    let expires_at = req.ttl_seconds.map(|s| now + chrono::Duration::seconds(s));

    let created_by = created_by_override
        .map(|s| s.to_string())
        .or_else(|| req.created_by.clone())
        .unwrap_or_else(|| "unknown".into());
    let confidence = confidence_override.or(req.confidence).unwrap_or(1.0);
    let write_source = write_source_override
        .map(|s| s.to_string())
        .or_else(|| req.source.clone())
        .unwrap_or_else(|| "api".into());

    let memory_type = memory_type_override
        .map(|s| s.to_string())
        .or_else(|| req.memory_type.clone())
        .unwrap_or_else(|| "core".into());

    let row: MemoryRow = sqlx::query_as(
        &format!("INSERT INTO memories (id, external_id, tenant_id, agent_id, namespace, content, tags, metadata, expires_at, created_at, updated_at, is_compressed, merged_from, created_by, confidence, write_source, memory_type) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $10, FALSE, '{{}}', $11, $12, $13, $14) RETURNING {}", MEMORY_COLUMNS)
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
    .bind(&created_by)
    .bind(confidence)
    .bind(&write_source)
    .bind(&memory_type)
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
        &format!("SELECT {} FROM memories WHERE tenant_id = $1 AND ($2::TEXT IS NULL OR namespace = $2) AND ($3::TEXT IS NULL OR agent_id = $3) ORDER BY created_at DESC LIMIT $4 OFFSET $5", MEMORY_COLUMNS)
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
        &format!("SELECT {} FROM memories WHERE tenant_id = $1 AND external_id = $2", MEMORY_COLUMNS)
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
    last_modified_by: Option<&str>,
) -> Result<Option<MemoryRow>, AppError> {
    let now = Utc::now();
    let row: Option<MemoryRow> = sqlx::query_as(
        &format!("UPDATE memories SET \
         content = COALESCE($3, content), \
         tags = COALESCE($4, tags), \
         metadata = COALESCE($5, metadata), \
         updated_at = $6, \
         last_modified_by = COALESCE($7, last_modified_by) \
         WHERE tenant_id = $1 AND external_id = $2 \
         RETURNING {}", MEMORY_COLUMNS)
    )
    .bind(tenant_id)
    .bind(external_id)
    .bind(content)
    .bind(tags)
    .bind(metadata)
    .bind(now)
    .bind(last_modified_by)
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

/// Bulk delete outdated memories scoped to a specific created_by (for append_only policy).
pub async fn delete_outdated_scoped(
    db: &PgPool,
    tenant_id: Uuid,
    older_than: Option<DateTime<Utc>>,
    tags: Option<&[String]>,
    namespace: Option<&str>,
    agent_id: Option<&str>,
    created_by_filter: &str,
    dry_run: bool,
) -> Result<usize, AppError> {
    if dry_run {
        let (count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM memories WHERE tenant_id = $1 \
             AND ($2::TIMESTAMPTZ IS NULL OR created_at < $2) \
             AND ($3::TEXT[] IS NULL OR tags @> $3) \
             AND ($4::TEXT IS NULL OR namespace = $4) \
             AND ($5::TEXT IS NULL OR agent_id = $5) \
             AND created_by = $6"
        )
        .bind(tenant_id)
        .bind(older_than)
        .bind(tags)
        .bind(namespace)
        .bind(agent_id)
        .bind(created_by_filter)
        .fetch_one(db)
        .await?;
        Ok(count as usize)
    } else {
        let result = sqlx::query(
            "DELETE FROM memories WHERE tenant_id = $1 \
             AND ($2::TIMESTAMPTZ IS NULL OR created_at < $2) \
             AND ($3::TEXT[] IS NULL OR tags @> $3) \
             AND ($4::TEXT IS NULL OR namespace = $4) \
             AND ($5::TEXT IS NULL OR agent_id = $5) \
             AND created_by = $6"
        )
        .bind(tenant_id)
        .bind(older_than)
        .bind(tags)
        .bind(namespace)
        .bind(agent_id)
        .bind(created_by_filter)
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

/// Delete provisional memories that have exceeded their tenant's TTL.
pub async fn expire_provisional_memories(db: &PgPool) -> Result<Vec<String>, AppError> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "DELETE FROM memories m USING tenants t \
         WHERE m.tenant_id = t.id \
         AND m.memory_type = 'provisional' \
         AND m.created_at + (COALESCE(t.provisional_ttl_days, 7) * INTERVAL '1 day') < NOW() \
         RETURNING m.external_id"
    )
    .fetch_all(db)
    .await?;
    Ok(rows.into_iter().map(|(eid,)| eid).collect())
}

/// Update a memory's memory_type. Returns the updated row.
pub async fn update_memory_type(
    db: &PgPool,
    tenant_id: Uuid,
    external_id: &str,
    memory_type: &str,
) -> Result<Option<MemoryRow>, AppError> {
    let now = Utc::now();
    let row: Option<MemoryRow> = sqlx::query_as(
        &format!("UPDATE memories SET memory_type = $3, updated_at = $4 \
         WHERE tenant_id = $1 AND external_id = $2 \
         RETURNING {}", MEMORY_COLUMNS)
    )
    .bind(tenant_id)
    .bind(external_id)
    .bind(memory_type)
    .bind(now)
    .fetch_optional(db)
    .await?;
    Ok(row)
}

/// Update a memory's content and memory_type. Returns the updated row.
pub async fn update_memory_content_and_type(
    db: &PgPool,
    tenant_id: Uuid,
    external_id: &str,
    content: &str,
    memory_type: &str,
) -> Result<Option<MemoryRow>, AppError> {
    let now = Utc::now();
    let row: Option<MemoryRow> = sqlx::query_as(
        &format!("UPDATE memories SET content = $3, memory_type = $4, updated_at = $5 \
         WHERE tenant_id = $1 AND external_id = $2 \
         RETURNING {}", MEMORY_COLUMNS)
    )
    .bind(tenant_id)
    .bind(external_id)
    .bind(content)
    .bind(memory_type)
    .bind(now)
    .fetch_optional(db)
    .await?;
    Ok(row)
}

/// Get tenant's judge_model setting.
pub async fn get_tenant_judge_model(db: &PgPool, tenant_id: Uuid) -> Result<String, AppError> {
    let row: (Option<String>,) = sqlx::query_as(
        "SELECT judge_model FROM tenants WHERE id = $1"
    )
    .bind(tenant_id)
    .fetch_one(db)
    .await?;
    Ok(row.0.unwrap_or_else(|| "gpt-4o-mini".into()))
}
