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

    // Enforce max memories per namespace based on plan.
    let max_memories = match tenant.plan.as_str() {
        "pro" => state.config.max_memories_pro,
        _ => state.config.max_memories_starter,
    };
    let current_count =
        db::count_memories_in_namespace(&state.db, tenant.tenant_id, &body.namespace).await?;
    if current_count >= max_memories {
        return Err(AppError::BadRequest(format!(
            "namespace '{}' has reached the {} memory limit ({})",
            body.namespace, tenant.plan, max_memories
        )));
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
                is_compressed: row.is_compressed,
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

/// POST /v1/memories/prune — manually trigger expired memory pruning.
async fn prune_memories(
    _tenant: TenantContext,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let pruned = db::prune_expired_memories(&state.db).await?;
    let count = pruned.len();
    tracing::info!("Manual prune: removed {} expired memories", count);
    Ok(Json(serde_json::json!({ "pruned": count })))
}

/// PATCH /v1/memories/{id} — update content, tags, or metadata.
async fn update_memory_handler(
    tenant: TenantContext,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateMemoryRequest>,
) -> Result<Json<MemoryResponse>, AppError> {
    if body.content.is_none() && body.tags.is_none() && body.metadata.is_none() {
        return Err(AppError::BadRequest(
            "at least one of content, tags, or metadata must be provided".into(),
        ));
    }
    if let Some(ref c) = body.content {
        if c.is_empty() {
            return Err(AppError::BadRequest("content must not be empty".into()));
        }
        if c.len() > 8192 {
            return Err(AppError::BadRequest("content exceeds 8192 char limit".into()));
        }
    }
    if let Some(ref tags) = body.tags {
        if tags.len() > 20 {
            return Err(AppError::BadRequest("max 20 tags allowed".into()));
        }
    }

    let row = db::update_memory(
        &state.db,
        tenant.tenant_id,
        &id,
        body.content.as_deref(),
        body.tags.as_deref(),
        body.metadata.as_ref(),
    )
    .await?
    .ok_or(AppError::NotFound)?;

    // Re-embed if content changed.
    if body.content.is_some() && !state.config.openai_api_key.is_empty() {
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
                        tracing::warn!("Qdrant re-embed failed for {}: {}", external_id, e);
                    }
                }
                Err(e) => {
                    tracing::warn!("Re-embedding failed for {}: {}", external_id, e);
                }
            }
        });
    }

    let response = row.to_response();
    state.ws.broadcast(MemoryEvent {
        event_type: "memory.updated".into(),
        memory: serde_json::to_value(&response).ok(),
        memory_id: Some(response.id.clone()),
        namespace: Some(response.namespace.clone()),
        agent_id: Some(response.agent_id.clone()),
        tenant_id: tenant.tenant_id,
    });

    Ok(Json(response))
}

/// DELETE /v1/memories/outdated — bulk delete by age, tags, namespace.
async fn delete_outdated_handler(
    tenant: TenantContext,
    State(state): State<AppState>,
    Json(body): Json<DeleteOutdatedRequest>,
) -> Result<Json<DeleteOutdatedResponse>, AppError> {
    if body.older_than_seconds.is_none() && body.tags.is_none() {
        return Err(AppError::BadRequest(
            "at least one of older_than_seconds or tags must be provided".into(),
        ));
    }

    let older_than = body
        .older_than_seconds
        .map(|s| chrono::Utc::now() - chrono::Duration::seconds(s));

    let deleted = db::delete_outdated(
        &state.db,
        tenant.tenant_id,
        older_than,
        body.tags.as_deref(),
        body.namespace.as_deref(),
        body.agent_id.as_deref(),
        body.dry_run,
    )
    .await?;

    Ok(Json(DeleteOutdatedResponse {
        deleted,
        dry_run: body.dry_run,
    }))
}

/// POST /v1/memories/merge — merge multiple memories into one.
async fn merge_memories_handler(
    tenant: TenantContext,
    State(state): State<AppState>,
    Json(body): Json<MergeMemoriesRequest>,
) -> Result<(StatusCode, Json<MemoryResponse>), AppError> {
    if body.memory_ids.len() < 2 {
        return Err(AppError::BadRequest(
            "at least 2 memory_ids required to merge".into(),
        ));
    }
    if body.memory_ids.len() > 50 {
        return Err(AppError::BadRequest("max 50 memories per merge".into()));
    }

    // Fetch all source memories.
    let mut sources = Vec::new();
    for mid in &body.memory_ids {
        let row = db::get_memory_by_external_id(&state.db, tenant.tenant_id, mid)
            .await?
            .ok_or_else(|| AppError::BadRequest(format!("memory '{}' not found", mid)))?;
        sources.push(row);
    }

    // Determine merged content.
    let merged_content = if let Some(ref c) = body.content {
        c.clone()
    } else {
        // LLM summarization.
        let texts: Vec<&str> = sources.iter().map(|s| s.content.as_str()).collect();
        let prompt = format!(
            "Merge these {} memories into a single concise memory that preserves all important information:\n\n{}",
            texts.len(),
            texts.iter().enumerate().map(|(i, t)| format!("{}. {}", i + 1, t)).collect::<Vec<_>>().join("\n")
        );
        crate::llm::call_llm(&state.config.openai_api_key, "gpt-4o-mini", &prompt)
            .await
            .map_err(|e| AppError::Internal(e))?
    };

    // Union tags or use provided.
    let merged_tags = if let Some(tags) = body.tags {
        tags
    } else {
        let mut all_tags: Vec<String> = sources.iter().flat_map(|s| s.tags.clone()).collect();
        all_tags.sort();
        all_tags.dedup();
        all_tags
    };

    let namespace = body
        .namespace
        .unwrap_or_else(|| sources[0].namespace.clone());
    let agent_id = body
        .agent_id
        .unwrap_or_else(|| sources[0].agent_id.clone());

    // Create merged memory.
    let create_req = CreateMemoryRequest {
        content: merged_content,
        tags: merged_tags,
        namespace,
        agent_id: Some(agent_id),
        metadata: serde_json::json!({"merged_from": body.memory_ids}),
        ttl_seconds: None,
    };
    let new_row = db::insert_memory(&state.db, tenant.tenant_id, &create_req).await?;

    // Mark as compressed/merged.
    sqlx::query(
        "UPDATE memories SET is_compressed = TRUE, merged_from = $1 WHERE external_id = $2 AND tenant_id = $3"
    )
    .bind(&body.memory_ids)
    .bind(&new_row.external_id)
    .bind(tenant.tenant_id)
    .execute(&state.db)
    .await
    .ok();

    // Embed the new memory.
    if !state.config.openai_api_key.is_empty() {
        let embed_state = state.clone();
        let content = new_row.content.clone();
        let external_id = new_row.external_id.clone();
        let tenant_id_str = new_row.tenant_id.to_string();
        let ns = new_row.namespace.clone();
        let aid = new_row.agent_id.clone();
        tokio::spawn(async move {
            match embed_state.get_embedding(&content).await {
                Ok(vector) => {
                    let point_id = uuid::Uuid::new_v4().to_string();
                    let payload = serde_json::json!({
                        "tenant_id": tenant_id_str,
                        "external_id": external_id,
                        "namespace": ns,
                        "agent_id": aid,
                    });
                    let _ = embed_state.qdrant_upsert(&point_id, vector, payload).await;
                }
                Err(_) => {}
            }
        });
    }

    // Delete source memories.
    for src in &sources {
        let _ = db::delete_memory(&state.db, tenant.tenant_id, &src.external_id).await;
    }

    let response = new_row.to_response();
    Ok((StatusCode::CREATED, Json(response)))
}

pub fn memory_routes() -> Router<AppState> {
    Router::new()
        .route("/v1/memories", post(create_memory).get(list_memories))
        .route("/v1/memories/recall", get(recall_memories))
        .route("/v1/memories/prune", post(prune_memories))
        .route("/v1/memories/outdated", delete(delete_outdated_handler))
        .route("/v1/memories/merge", post(merge_memories_handler))
        .route(
            "/v1/memories/{id}",
            delete(delete_memory).patch(update_memory_handler),
        )
}
