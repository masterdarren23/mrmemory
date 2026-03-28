use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::TenantContext;
use crate::db::memories as db;
use crate::error::AppError;
use crate::llm::{self, ChatMessage};
use crate::models::CreateMemoryRequest;
use crate::state::AppState;
use crate::ws::MemoryEvent;

#[derive(Debug, Deserialize)]
pub struct AutoRememberRequest {
    pub messages: Vec<ChatMessage>,
    #[serde(default = "default_namespace")]
    pub namespace: String,
    pub agent_id: Option<String>,
    /// BYOK: caller-provided OpenAI key
    pub llm_api_key: Option<String>,
    #[serde(default = "default_model")]
    pub llm_model: String,
    #[serde(default)]
    pub sync: bool,
}

fn default_namespace() -> String {
    "default".into()
}
fn default_model() -> String {
    "gpt-4o-mini".into()
}

#[derive(Debug, Serialize)]
pub struct AutoRememberSyncResponse {
    pub memories: Vec<AutoRememberMemoryResult>,
    pub extracted: usize,
    pub created: usize,
    pub duplicates_skipped: usize,
    pub updated: usize,
}

#[derive(Debug, Serialize)]
pub struct AutoRememberMemoryResult {
    pub id: String,
    pub content: String,
    pub action: String, // "created", "skipped", "updated"
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub similarity: Option<f32>,
}

/// POST /v1/memories/auto
pub async fn auto_remember(
    tenant: TenantContext,
    State(state): State<AppState>,
    Json(body): Json<AutoRememberRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    if body.messages.is_empty() {
        return Err(AppError::BadRequest("messages must not be empty".into()));
    }
    if body.messages.len() > 100 {
        return Err(AppError::BadRequest("max 100 messages allowed".into()));
    }

    // Resolve API key: BYOK first, then env
    let api_key = body
        .llm_api_key
        .clone()
        .unwrap_or_else(|| state.config.openai_api_key.clone());

    if api_key.is_empty() {
        return Err(AppError::BadRequest(
            "no OpenAI API key available; provide llm_api_key or set OPENAI_API_KEY".into(),
        ));
    }

    if body.sync {
        // Synchronous mode: block and return results
        let results = process_auto_remember(
            &state,
            tenant.tenant_id,
            &body.messages,
            &body.namespace,
            body.agent_id.as_deref(),
            &body.llm_model,
            &api_key,
        )
        .await
        .map_err(|e| AppError::Internal(e))?;

        // Broadcast completion event
        state.ws.broadcast(MemoryEvent {
            event_type: "memory.auto_completed".into(),
            memory: Some(serde_json::to_value(&results).unwrap_or_default()),
            memory_id: None,
            namespace: Some(body.namespace.clone()),
            agent_id: body.agent_id.clone(),
            tenant_id: tenant.tenant_id,
        });

        Ok((
            StatusCode::OK,
            Json(serde_json::to_value(&results).unwrap()),
        ))
    } else {
        // Async mode: return job_id immediately, process in background
        let job_id = format!("job_{}", Uuid::new_v4().simple());

        let async_state = state.clone();
        let tenant_id = tenant.tenant_id;
        let messages = body.messages.clone();
        let namespace = body.namespace.clone();
        let agent_id = body.agent_id.clone();
        let model = body.llm_model.clone();
        let job_id_clone = job_id.clone();

        tokio::spawn(async move {
            match process_auto_remember(
                &async_state,
                tenant_id,
                &messages,
                &namespace,
                agent_id.as_deref(),
                &model,
                &api_key,
            )
            .await
            {
                Ok(results) => {
                    tracing::info!(
                        "Auto-remember job {} completed: {} created, {} skipped",
                        job_id_clone,
                        results.created,
                        results.duplicates_skipped
                    );
                    async_state.ws.broadcast(MemoryEvent {
                        event_type: "memory.auto_completed".into(),
                        memory: Some(serde_json::json!({
                            "job_id": job_id_clone,
                            "results": results,
                        })),
                        memory_id: None,
                        namespace: Some(namespace),
                        agent_id,
                        tenant_id,
                    });
                }
                Err(e) => {
                    tracing::error!("Auto-remember job {} failed: {}", job_id_clone, e);
                    async_state.ws.broadcast(MemoryEvent {
                        event_type: "memory.auto_failed".into(),
                        memory: Some(serde_json::json!({
                            "job_id": job_id_clone,
                            "error": e.to_string(),
                        })),
                        memory_id: None,
                        namespace: Some(namespace),
                        agent_id,
                        tenant_id,
                    });
                }
            }
        });

        Ok((
            StatusCode::ACCEPTED,
            Json(serde_json::json!({
                "job_id": job_id,
                "status": "processing"
            })),
        ))
    }
}

/// Core logic: extract memories via LLM, deduplicate, and store.
async fn process_auto_remember(
    state: &AppState,
    tenant_id: Uuid,
    messages: &[ChatMessage],
    namespace: &str,
    agent_id: Option<&str>,
    model: &str,
    api_key: &str,
) -> anyhow::Result<AutoRememberSyncResponse> {
    // Step 1: Extract memories via LLM
    let extracted = llm::extract_memories(&state.http, messages, model, api_key).await?;

    let mut results = Vec::new();
    let mut created = 0usize;
    let mut duplicates_skipped = 0usize;
    let mut updated = 0usize;

    // Step 2: For each extracted memory, deduplicate and insert
    for fact in &extracted {
        // Get embedding for deduplication search
        let embedding_result = state.get_embedding(&fact.content).await;

        let mut action = "created";
        let mut similarity: Option<f32> = None;
        let mut memory_id = String::new();

        if let Ok(vector) = &embedding_result {
            // Build Qdrant filter for tenant + namespace
            let mut must = vec![serde_json::json!({
                "key": "tenant_id",
                "match": { "value": tenant_id.to_string() }
            })];
            must.push(serde_json::json!({
                "key": "namespace",
                "match": { "value": namespace }
            }));
            if let Some(aid) = agent_id {
                must.push(serde_json::json!({
                    "key": "agent_id",
                    "match": { "value": aid }
                }));
            }
            let filter = serde_json::json!({ "must": must });

            // Search for similar existing memories
            if let Ok(hits) = state
                .qdrant_search(vector.clone(), 1, 0.7, Some(filter))
                .await
            {
                if let Some((existing_id, score)) = hits.first() {
                    similarity = Some(*score);

                    if *score > 0.9 {
                        // Very similar — skip (duplicate)
                        action = "skipped";
                        duplicates_skipped += 1;
                        memory_id = existing_id.clone();

                        results.push(AutoRememberMemoryResult {
                            id: memory_id,
                            content: fact.content.clone(),
                            action: action.into(),
                            tags: fact.tags.clone(),
                            similarity,
                        });
                        continue;
                    } else {
                        // 0.7–0.9: update existing memory
                        // For now, we update by deleting old and inserting new
                        if let Ok(Some(_)) =
                            db::get_memory_by_external_id(&state.db, tenant_id, existing_id).await
                        {
                            let _ =
                                db::delete_memory(&state.db, tenant_id, existing_id).await;
                            action = "updated";
                            updated += 1;
                        }
                    }
                }
            }
        }

        // Insert the memory
        let create_req = CreateMemoryRequest {
            content: fact.content.clone(),
            tags: fact.tags.clone(),
            namespace: namespace.to_string(),
            agent_id: agent_id.map(|s| s.to_string()),
            metadata: serde_json::json!({
                "source": "auto_remember",
                "importance": fact.importance,
            }),
            ttl_seconds: fact.suggested_ttl_seconds,
            validate: None,
            created_by: None,
            confidence: None,
            source: None,
            memory_type: None,
        };

        match db::insert_memory_with_provenance(&state.db, tenant_id, &create_req, None, None, Some("auto_remember"), None).await {
            Ok(row) => {
                memory_id = row.external_id.clone();

                if action == "created" {
                    created += 1;
                }

                // Upsert embedding to Qdrant
                if let Ok(vector) = &embedding_result {
                    let point_id = Uuid::new_v4().to_string();
                    let payload = serde_json::json!({
                        "tenant_id": tenant_id.to_string(),
                        "external_id": row.external_id,
                        "namespace": namespace,
                        "agent_id": agent_id.unwrap_or("default"),
                    });
                    if let Err(e) = state.qdrant_upsert(&point_id, vector.clone(), payload).await {
                        tracing::warn!("Qdrant upsert failed for {}: {}", row.external_id, e);
                    }
                }

                // Broadcast individual memory.created event
                let response = row.to_response();
                state.ws.broadcast(MemoryEvent {
                    event_type: "memory.created".into(),
                    memory: serde_json::to_value(&response).ok(),
                    memory_id: Some(response.id.clone()),
                    namespace: Some(response.namespace.clone()),
                    agent_id: Some(response.agent_id.clone()),
                    tenant_id,
                });
            }
            Err(e) => {
                tracing::warn!("Failed to insert auto-remember memory: {}", e);
                continue;
            }
        }

        results.push(AutoRememberMemoryResult {
            id: memory_id,
            content: fact.content.clone(),
            action: action.into(),
            tags: fact.tags.clone(),
            similarity,
        });
    }

    Ok(AutoRememberSyncResponse {
        extracted: extracted.len(),
        memories: results,
        created,
        duplicates_skipped,
        updated,
    })
}
