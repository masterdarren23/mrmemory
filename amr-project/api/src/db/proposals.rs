use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, sqlx::FromRow)]
pub struct ProposalRow {
    pub id: Uuid,
    pub external_id: String,
    pub tenant_id: Uuid,
    pub memory_id: Uuid,
    pub proposed_by: String,
    pub justification: String,
    pub evidence_ids: Vec<String>,
    pub confidence: Option<f32>,
    pub status: String,
    pub judge_model: Option<String>,
    pub judge_decision: Option<serde_json::Value>,
    pub decided_at: Option<DateTime<Utc>>,
    pub decided_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

const PROPOSAL_COLUMNS: &str = "id, external_id, tenant_id, memory_id, proposed_by, justification, evidence_ids, confidence, status, judge_model, judge_decision, decided_at, decided_by, created_at, expires_at";

pub async fn insert_proposal(
    db: &PgPool,
    tenant_id: Uuid,
    memory_id: Uuid,
    proposed_by: &str,
    justification: &str,
    evidence_ids: &[String],
    confidence: Option<f32>,
    expires_at: Option<DateTime<Utc>>,
) -> Result<ProposalRow, AppError> {
    let id = Uuid::new_v4();
    let external_id = format!(
        "prop_{}",
        id.simple().to_string().get(..12).unwrap_or("000000000000")
    );

    let row: ProposalRow = sqlx::query_as(
        &format!(
            "INSERT INTO memory_proposals (id, external_id, tenant_id, memory_id, proposed_by, justification, evidence_ids, confidence, expires_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
             RETURNING {}",
            PROPOSAL_COLUMNS
        ),
    )
    .bind(id)
    .bind(&external_id)
    .bind(tenant_id)
    .bind(memory_id)
    .bind(proposed_by)
    .bind(justification)
    .bind(evidence_ids)
    .bind(confidence.unwrap_or(0.8))
    .bind(expires_at)
    .fetch_one(db)
    .await?;

    Ok(row)
}

pub async fn get_proposal(
    db: &PgPool,
    tenant_id: Uuid,
    external_id: &str,
) -> Result<Option<ProposalRow>, AppError> {
    let row: Option<ProposalRow> = sqlx::query_as(
        &format!(
            "SELECT {} FROM memory_proposals WHERE tenant_id = $1 AND external_id = $2",
            PROPOSAL_COLUMNS
        ),
    )
    .bind(tenant_id)
    .bind(external_id)
    .fetch_optional(db)
    .await?;

    Ok(row)
}

pub async fn list_proposals(
    db: &PgPool,
    tenant_id: Uuid,
    status: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<ProposalRow>, AppError> {
    let rows: Vec<ProposalRow> = sqlx::query_as(
        &format!(
            "SELECT {} FROM memory_proposals \
             WHERE tenant_id = $1 \
             AND ($2::TEXT IS NULL OR status = $2) \
             ORDER BY created_at DESC LIMIT $3 OFFSET $4",
            PROPOSAL_COLUMNS
        ),
    )
    .bind(tenant_id)
    .bind(status)
    .bind(limit)
    .bind(offset)
    .fetch_all(db)
    .await?;

    Ok(rows)
}

pub async fn decide_proposal(
    db: &PgPool,
    tenant_id: Uuid,
    external_id: &str,
    status: &str,
    decided_by: &str,
    judge_model: Option<&str>,
    judge_decision: Option<&serde_json::Value>,
) -> Result<Option<ProposalRow>, AppError> {
    let now = Utc::now();
    let row: Option<ProposalRow> = sqlx::query_as(
        &format!(
            "UPDATE memory_proposals SET \
             status = $3, decided_by = $4, decided_at = $5, \
             judge_model = COALESCE($6, judge_model), \
             judge_decision = COALESCE($7, judge_decision) \
             WHERE tenant_id = $1 AND external_id = $2 AND status = 'pending' \
             RETURNING {}",
            PROPOSAL_COLUMNS
        ),
    )
    .bind(tenant_id)
    .bind(external_id)
    .bind(status)
    .bind(decided_by)
    .bind(now)
    .bind(judge_model)
    .bind(judge_decision)
    .fetch_optional(db)
    .await?;

    Ok(row)
}

/// Get the memory external_id for a proposal's memory_id.
pub async fn get_memory_external_id(
    db: &PgPool,
    memory_id: Uuid,
) -> Result<Option<String>, AppError> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT external_id FROM memories WHERE id = $1",
    )
    .bind(memory_id)
    .fetch_optional(db)
    .await?;
    Ok(row.map(|(eid,)| eid))
}
