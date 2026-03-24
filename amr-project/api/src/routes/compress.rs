use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::TenantContext;
use crate::db::memories as db;
use crate::error::AppError;
use crate::models::CreateMemoryRequest;
use crate::state::AppState;
use crate::ws::MemoryEvent;

#[derive(Debug, Deserialize)]
pub struct CompressRequest {
    #[serde(default = "default_namespace")]
    pub namespace: String,
    pub agent_id: Option<String>,
    /// BYOK: caller-provided OpenAI key
    pub llm_api_key: Option<String>,
    #[serde(default = "default_model")]
    pub llm_model: String,
    /// Minimum memories before compression triggers (default 10)
    #[serde(default = "default_threshold")]
    pub threshold: i64,
    /// Target number of memories after compression (default: threshold / 2)
    pub target_count: Option<i64>,
    /// Similarity threshold for grouping memories (default 0.75)
    #[serde(default = "default_similarity")]
    pub similarity_threshold: f32,
    #[serde(default)]
    pub sync: bool,
    /// Dry run: return what would be compressed without actually doing it
    #[serde(default)]
    pub dry_run: bool,
}

fn default_namespace() -> String { "default".into() }
fn default_model() -> String { "gpt-4o-mini".into() }
fn default_threshold() -> i64 { 10 }
fn default_similarity() -> f32 { 0.75 }

#[derive(Debug, Serialize)]
pub struct CompressResponse {
    pub status: String,
    pub namespace: String,
    pub before_count: i64,
    pub after_count: i64,
    pub groups_compressed: usize,
    pub memories_removed: usize,
    pub memories_created: usize,
    pub groups: Vec<CompressedGroup>,
}

#[derive(Debug, Serialize, Clone)]
pub struct CompressedGroup {
    pub original_ids: Vec<String>,
    pub original_contents: Vec<String>,
    pub compressed_content: String,
    pub compressed_id: Option<String>,
    pub tags: Vec<String>,
}

/// POST /v1/memories/compress
pub async fn compress_memories(
    tenant: TenantContext,
    State(state): State<AppState>,
    Json(body): Json<CompressRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let api_key = body
        .llm_api_key
        .clone()
        .unwrap_or_else(|| state.config.openai_api_key.clone());

    if api_key.is_empty() {
        return Err(AppError::BadRequest(
            "no OpenAI API key available; provide llm_api_key or set OPENAI_API_KEY".into(),
        ));
    }

    // Check memory count
    let count = db::count_memories_in_namespace(&state.db, tenant.tenant_id, &body.namespace).await?;

    if count < body.threshold {
        return Ok((
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "skipped",
                "namespace": body.namespace,
                "reason": format!("only {} memories (threshold: {})", count, body.threshold),
                "before_count": count,
                "after_count": count,
                "groups_compressed": 0,
                "memories_removed": 0,
                "memories_created": 0,
                "groups": []
            })),
        ));
    }

    if body.sync {
        let result = process_compression(
            &state,
            tenant.tenant_id,
            &body.namespace,
            body.agent_id.as_deref(),
            &body.llm_model,
            &api_key,
            body.similarity_threshold,
            body.target_count,
            body.dry_run,
        )
        .await
        .map_err(AppError::Internal)?;

        state.ws.broadcast(MemoryEvent {
            event_type: "memory.compress_completed".into(),
            memory: Some(serde_json::to_value(&result).unwrap_or_default()),
            memory_id: None,
            namespace: Some(body.namespace.clone()),
            agent_id: body.agent_id.clone(),
            tenant_id: tenant.tenant_id,
        });

        Ok((StatusCode::OK, Json(serde_json::to_value(&result).unwrap())))
    } else {
        let job_id = format!("compress_{}", Uuid::new_v4().simple());
        let async_state = state.clone();
        let tenant_id = tenant.tenant_id;
        let namespace = body.namespace.clone();
        let agent_id = body.agent_id.clone();
        let model = body.llm_model.clone();
        let sim_thresh = body.similarity_threshold;
        let target = body.target_count;
        let dry_run = body.dry_run;
        let job_id_clone = job_id.clone();

        tokio::spawn(async move {
            match process_compression(
                &async_state, tenant_id, &namespace, agent_id.as_deref(),
                &model, &api_key, sim_thresh, target, dry_run,
            ).await {
                Ok(result) => {
                    tracing::info!(
                        "Compress job {} completed: {} groups, {} removed, {} created",
                        job_id_clone, result.groups_compressed, result.memories_removed, result.memories_created
                    );
                    async_state.ws.broadcast(MemoryEvent {
                        event_type: "memory.compress_completed".into(),
                        memory: Some(serde_json::json!({
                            "job_id": job_id_clone,
                            "results": result,
                        })),
                        memory_id: None,
                        namespace: Some(namespace),
                        agent_id,
                        tenant_id,
                    });
                }
                Err(e) => {
                    tracing::error!("Compress job {} failed: {}", job_id_clone, e);
                    async_state.ws.broadcast(MemoryEvent {
                        event_type: "memory.compress_failed".into(),
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
                "status": "processing",
                "memory_count": count
            })),
        ))
    }
}

/// Core compression logic.
async fn process_compression(
    state: &AppState,
    tenant_id: Uuid,
    namespace: &str,
    agent_id: Option<&str>,
    model: &str,
    api_key: &str,
    similarity_threshold: f32,
    _target_count: Option<i64>,
    dry_run: bool,
) -> anyhow::Result<CompressResponse> {
    // Step 1: Fetch all memories in namespace
    let (memories, total) = db::list_memories(
        &state.db, tenant_id, Some(namespace), agent_id, 500, 0,
    ).await?;

    if memories.is_empty() {
        return Ok(CompressResponse {
            status: "skipped".into(),
            namespace: namespace.into(),
            before_count: 0,
            after_count: 0,
            groups_compressed: 0,
            memories_removed: 0,
            memories_created: 0,
            groups: vec![],
        });
    }

    // Step 2: Get embeddings for all memories and group by similarity
    let mut embeddings: Vec<(usize, Vec<f32>)> = Vec::new();
    for (i, mem) in memories.iter().enumerate() {
        match state.get_embedding(&mem.content).await {
            Ok(emb) => embeddings.push((i, emb)),
            Err(e) => {
                tracing::warn!("Failed to embed memory {}: {}", mem.external_id, e);
            }
        }
    }

    // Step 3: Greedy clustering by cosine similarity
    let mut assigned = vec![false; memories.len()];
    let mut groups: Vec<Vec<usize>> = Vec::new();

    for &(i, ref emb_i) in &embeddings {
        if assigned[i] { continue; }
        assigned[i] = true;
        let mut group = vec![i];

        for &(j, ref emb_j) in &embeddings {
            if assigned[j] { continue; }
            let sim = cosine_similarity(emb_i, emb_j);
            if sim >= similarity_threshold {
                assigned[j] = true;
                group.push(j);
            }
        }

        groups.push(group);
    }

    // Also add any memories that failed embedding as singletons
    for (i, _mem) in memories.iter().enumerate() {
        if !assigned[i] {
            groups.push(vec![i]);
        }
    }

    // Step 4: Only compress groups with 2+ memories
    let groups_to_compress: Vec<Vec<usize>> = groups.into_iter()
        .filter(|g| g.len() >= 2)
        .collect();

    if groups_to_compress.is_empty() {
        return Ok(CompressResponse {
            status: "skipped".into(),
            namespace: namespace.into(),
            before_count: total,
            after_count: total,
            groups_compressed: 0,
            memories_removed: 0,
            memories_created: 0,
            groups: vec![],
        });
    }

    // Step 5: For each group, call LLM to compress
    let mut compressed_groups: Vec<CompressedGroup> = Vec::new();
    let mut total_removed = 0usize;
    let mut total_created = 0usize;

    for group_indices in &groups_to_compress {
        let original_contents: Vec<String> = group_indices.iter()
            .map(|&i| memories[i].content.clone())
            .collect();
        let original_ids: Vec<String> = group_indices.iter()
            .map(|&i| memories[i].external_id.clone())
            .collect();

        // Collect all tags from the group
        let mut all_tags: Vec<String> = group_indices.iter()
            .flat_map(|&i| memories[i].tags.clone())
            .collect();
        all_tags.sort();
        all_tags.dedup();

        // LLM compression call
        let compressed_content = compress_via_llm(
            &state.http, &original_contents, model, api_key,
        ).await?;

        let mut compressed_id = None;

        if !dry_run {
            // Delete originals
            for &i in group_indices {
                let _ = db::delete_memory(&state.db, tenant_id, &memories[i].external_id).await;
                // TODO: also delete from Qdrant
            }
            total_removed += group_indices.len();

            // Insert compressed memory
            let create_req = CreateMemoryRequest {
                content: compressed_content.clone(),
                tags: all_tags.clone(),
                namespace: namespace.to_string(),
                agent_id: agent_id.map(|s| s.to_string()),
                metadata: serde_json::json!({
                    "source": "compression",
                    "compressed_from": original_ids,
                    "original_count": group_indices.len(),
                }),
                ttl_seconds: None,
            };

            match db::insert_memory(&state.db, tenant_id, &create_req).await {
                Ok(row) => {
                    compressed_id = Some(row.external_id.clone());
                    total_created += 1;

                    // Embed and upsert to Qdrant
                    if let Ok(vector) = state.get_embedding(&compressed_content).await {
                        let point_id = Uuid::new_v4().to_string();
                        let payload = serde_json::json!({
                            "tenant_id": tenant_id.to_string(),
                            "external_id": row.external_id,
                            "namespace": namespace,
                            "agent_id": agent_id.unwrap_or("default"),
                        });
                        let _ = state.qdrant_upsert(&point_id, vector, payload).await;
                    }

                    // Broadcast event
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
                    tracing::warn!("Failed to insert compressed memory: {}", e);
                }
            }
        }

        compressed_groups.push(CompressedGroup {
            original_ids,
            original_contents,
            compressed_content,
            compressed_id,
            tags: all_tags,
        });
    }

    let after_count = total - total_removed as i64 + total_created as i64;

    Ok(CompressResponse {
        status: if dry_run { "dry_run".into() } else { "completed".into() },
        namespace: namespace.into(),
        before_count: total,
        after_count,
        groups_compressed: compressed_groups.len(),
        memories_removed: total_removed,
        memories_created: total_created,
        groups: compressed_groups,
    })
}

/// Cosine similarity between two vectors.
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a * norm_b)
}

const COMPRESS_SYSTEM_PROMPT: &str = r#"You are a memory compression engine. Given a group of related memories, merge them into a single, concise memory that preserves all important facts.

Rules:
- Combine related facts into one clear, information-dense statement
- Preserve specific details (names, numbers, dates, preferences)
- Remove redundancy and filler
- Keep the tone neutral and factual
- Output ONLY the compressed memory text, nothing else
- If memories contradict each other, keep the most recent/specific version"#;

/// Call OpenAI to compress a group of memories into one.
async fn compress_via_llm(
    http: &reqwest::Client,
    memories: &[String],
    model: &str,
    api_key: &str,
) -> anyhow::Result<String> {
    let memory_list = memories.iter()
        .enumerate()
        .map(|(i, m)| format!("{}. {}", i + 1, m))
        .collect::<Vec<_>>()
        .join("\n");

    let messages = vec![
        serde_json::json!({ "role": "system", "content": COMPRESS_SYSTEM_PROMPT }),
        serde_json::json!({
            "role": "user",
            "content": format!("Compress these {} related memories into one:\n\n{}", memories.len(), memory_list)
        }),
    ];

    let body = serde_json::json!({
        "model": model,
        "messages": messages,
        "temperature": 0.1,
        "max_tokens": 500
    });

    let resp = http
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        anyhow::bail!("OpenAI API error {}: {}", status, text);
    }

    let json: serde_json::Value = resp.json().await?;
    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("no content in LLM response"))?
        .trim()
        .to_string();

    Ok(content)
}
