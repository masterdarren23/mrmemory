use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use std::time::Instant;

use crate::auth::TenantContext;
use crate::db::memories as db;
use crate::error::AppError;
use crate::models::*;
use crate::state::AppState;

/// POST /v1/memories — store a memory.
async fn create_memory(
    tenant: TenantContext,
    State(state): State<AppState>,
    Json(body): Json<CreateMemoryRequest>,
) -> Result<(StatusCode, Json<MemoryResponse>), AppError> {
    if body.content.is_empty() {
        return Err(AppError::BadRequest("content must not be empty".into()));
    }
    if body.content.len() > 8192 {
        return Err(AppError::BadRequest("content exceeds 8192 char limit".into()));
    }
    if body.tags.len() > 20 {
        return Err(AppError::BadRequest("max 20 tags allowed".into()));
    }
    for tag in &body.tags {
        if tag.len() > 64 {
            return Err(AppError::BadRequest(format!("tag '{}' exceeds 64 char limit", tag)));
        }
    }
    if body.namespace.len() > 128 {
        return Err(AppError::BadRequest("namespace exceeds 128 char limit".into()));
    }

    let row = db::insert_memory(&state.db, tenant.tenant_id, &body).await?;
    Ok((StatusCode::CREATED, Json(row.to_response())))
}

/// GET /v1/memories/recall — semantic search (stub: returns all with fake score).
async fn recall_memories(
    tenant: TenantContext,
    State(state): State<AppState>,
    Query(params): Query<RecallQuery>,
) -> Result<Json<RecallResponse>, AppError> {
    if params.query.is_empty() {
        return Err(AppError::BadRequest("query must not be empty".into()));
    }

    let start = Instant::now();

    // TODO: Replace with Qdrant vector search.
    // For now, fall back to listing recent memories as stub results.
    let limit = params.limit.min(100) as i64;
    let (rows, _total) = db::list_memories(
        &state.db,
        tenant.tenant_id,
        params.namespace.as_deref(),
        params.agent_id.as_deref(),
        limit,
        0,
    )
    .await?;

    let memories: Vec<RecallResult> = rows
        .iter()
        .map(|r| RecallResult {
            id: r.external_id.clone(),
            content: r.content.clone(),
            tags: r.tags.clone(),
            namespace: r.namespace.clone(),
            agent_id: r.agent_id.clone(),
            metadata: r.metadata.clone(),
            similarity: 0.85, // stub score until Qdrant is wired
            created_at: r.created_at,
        })
        .collect();

    let count = memories.len();
    let elapsed = start.elapsed().as_millis() as u64;

    Ok(Json(RecallResponse {
        memories,
        count,
        query_time_ms: elapsed,
    }))
}

/// GET /v1/memories — list with filters and pagination.
async fn list_memories(
    tenant: TenantContext,
    State(state): State<AppState>,
    Query(params): Query<ListMemoriesQuery>,
) -> Result<Json<ListMemoriesResponse>, AppError> {
    let limit = params.limit.min(100) as i64;
    let offset = params.offset as i64;

    let (rows, total) = db::list_memories(
        &state.db,
        tenant.tenant_id,
        params.namespace.as_deref(),
        params.agent_id.as_deref(),
        limit,
        offset,
    )
    .await?;

    Ok(Json(ListMemoriesResponse {
        memories: rows.iter().map(|r| r.to_response()).collect(),
        total: total as usize,
        limit: params.limit,
        offset: params.offset,
    }))
}

/// DELETE /v1/memories/{id} — forget a specific memory.
async fn delete_memory(
    tenant: TenantContext,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let deleted = db::delete_memory(&state.db, tenant.tenant_id, &id).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound)
    }
}

pub fn memory_routes() -> Router<AppState> {
    Router::new()
        .route("/v1/memories", post(create_memory).get(list_memories))
        .route("/v1/memories/recall", get(recall_memories))
        .route("/v1/memories/{id}", delete(delete_memory))
}
