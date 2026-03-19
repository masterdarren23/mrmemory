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
use crate::ws::MemoryEvent;

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
        return Err(AppError::BadRequest(
            "content exceeds 8192 char limit".into(),
        ));
    }
    if body.tags.len() > 20 {
        return Err(AppError::BadRequest("max 20 tags allowed".into()));
    }
    for tag in &body.tags {
        if tag.len() > 64 {
            return Err(AppError::BadRequest(format!(
                "tag '{}' exceeds 64 char limit",
                tag
            )));
        }
    }
    if body.namespace.len() > 128 {
        return Err(AppError::BadRequest(
            "namespace exceeds 128 char limit".into(),
        ));
    }

    let row = db::insert_memory(&state.db, tenant.tenant_id, &body).await?;

    // Generate embedding and upsert to Qdrant (best-effort, don't fail the request).
    if !state.config.openai_api_key.is_empty() {
        let embed_state = state.clone();
        let content = row.content.clone();
        let external_id = row.external_id.clone();
        let tenant_id_str = row.tenant_id.to_string();
        let namespace = row.namespace.clone();
        let agent_id = row.agent_id.clone();
        tokio::spawn(async move {
            match embed_state.get_embedding(&content).await {
                Ok(vector) => {
                    let point_id = uuid::Uuid::new_v4().to_string();
                    let payload = serde_json::json!({
                        "tenant_id": tenant_id_str,
                        "external_id": external_id,
                        "namespace": namespace,
                        "agent_id": agent_id,
                    });
                    if let Err(e) = embed_state.qdrant_upsert(&point_id, vector, payload).await {
                        tracing::warn!("Qdrant upsert failed for {}: {}", external_id, e);
                    }
                }
                Err(e) => {
                    tracing::warn!("Embedding generation failed for {}: {}", external_id, e);
                }
            }
        });
    }

    // Broadcast memory.created event to WebSocket subscribers.
    let response = row.to_response();
    state.ws.broadcast(MemoryEvent {
        event_type: "memory.created".into(),
        memory: serde_json::to_value(&response).ok(),
        memory_id: Some(response.id.clone()),
        namespace: Some(response.namespace.clone()),
        agent_id: Some(response.agent_id.clone()),
        tenant_id: tenant.tenant_id,
    });

    Ok((StatusCode::CREATED, Json(response)))
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
    let limit = params.limit.min(100);

    // Build Qdrant filter for tenant + optional namespace/agent_id.
    let mut must = vec![serde_json::json!({
        "key": "tenant_id",
        "match": { "value": tenant.tenant_id.to_string() }
    })];
    if let Some(ref ns) = params.namespace {
        must.push(serde_json::json!({
            "key": "namespace",
            "match": { "value": ns }
        }));
    }
    if let Some(ref aid) = params.agent_id {
        must.push(serde_json::json!({
            "key": "agent_id",
            "match": { "value": aid }
        }));
    }
    let filter = serde_json::json!({ "must": must });

    // Get embedding and search Qdrant.
    let vector = state
        .get_embedding(&params.query)
        .await
        .map_err(|e| AppError::Internal(e))?;

    let hits = state
        .qdrant_search(vector, limit, params.threshold, Some(filter))
        .await
        .map_err(|e| AppError::Internal(e))?;

    // Hydrate results from Postgres.
    let mut memories: Vec<RecallResult> = Vec::with_capacity(hits.len());
    for (external_id, score) in &hits {
        if let Some(row) =
            db::get_memory_by_external_id(&state.db, tenant.tenant_id, external_id).await?
        {
            memories.push(RecallResult {
                id: row.external_id.clone(),
                content: row.content.clone(),
                tags: row.tags.clone(),
                namespace: row.namespace.clone(),
                agent_id: row.agent_id.clone(),
                metadata: row.metadata.clone(),
                similarity: *score,
                created_at: row.created_at,
            });
        }
    }

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
    // Look up memory before deleting so we can broadcast with namespace/agent_id.
    let existing = db::get_memory_by_external_id(&state.db, tenant.tenant_id, &id).await?;

    let deleted = db::delete_memory(&state.db, tenant.tenant_id, &id).await?;
    if deleted {
        if let Some(mem) = existing {
            state.ws.broadcast(MemoryEvent {
                event_type: "memory.deleted".into(),
                memory: None,
                memory_id: Some(mem.external_id),
                namespace: Some(mem.namespace),
                agent_id: Some(mem.agent_id),
                tenant_id: tenant.tenant_id,
            });
        }
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
