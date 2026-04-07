use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};

use crate::auth::TenantContext;
use crate::db::memories as db;
use crate::db::proposals as proposals_db;
use crate::error::AppError;
use crate::models::*;
use crate::state::AppState;
use crate::ws::MemoryEvent;

const JUDGE_SYSTEM_PROMPT: &str = r#"You are an impartial truth-seeking judge for a multi-agent memory system.
Your ONLY job is to decide whether the proposed memory should be promoted from "provisional" to "core".
Core = permanent shared truth that all agents can rely on.

PROPOSED MEMORY:
{content}

JUSTIFICATION:
{justification}

EVIDENCE (from proposer's private/existing memories):
{evidence_contents}

EXISTING CORE MEMORIES IN THIS NAMESPACE:
{existing_core_contents}

Evaluate using these criteria (in order):
1. Does it increase our collective understanding of reality? (truth-adjacent)
2. Does it contradict any existing core memories? If yes, can it be merged cleanly?
3. Does it violate any hard safety/legal rules (e.g. PII leak, illegal content, harmful instructions)?
4. Is the justification and evidence sufficient?

Output ONLY a JSON object:
{"decision": "approve" | "reject" | "merge", "reason": "one short sentence", "suggested_merge_content": "optional rewritten version if merge", "new_tags": ["tag1", "tag2"]}

Be extremely truth-seeking and minimally restrictive. Only reject if it clearly harms truth or safety."#;

/// POST /v1/memories/private — store a private (ungoverned) memory.
async fn create_private_memory(
    tenant: TenantContext,
    State(state): State<AppState>,
    Json(body): Json<PrivateMemoryRequest>,
) -> Result<(StatusCode, Json<MemoryResponse>), AppError> {
    if body.content.is_empty() {
        return Err(AppError::BadRequest("content must not be empty".into()));
    }

    let ttl = body.ttl_seconds.unwrap_or(86400); // 24h default
    let create_req = CreateMemoryRequest {
        content: body.content,
        tags: body.tags,
        namespace: body.namespace,
        agent_id: body.agent_id,
        metadata: body.metadata,
        ttl_seconds: Some(ttl),
        validate: None,
        created_by: body.created_by,
        confidence: None,
        source: Some("private".into()),
        memory_type: Some("private".into()),
    };

    let row = db::insert_memory(&state.db, tenant.tenant_id, &create_req).await?;

    // Embed async
    if !state.config.openai_api_key.is_empty() {
        let s = state.clone();
        let content = row.content.clone();
        let eid = row.external_id.clone();
        let tid = row.tenant_id.to_string();
        let ns = row.namespace.clone();
        let aid = row.agent_id.clone();
        tokio::spawn(async move {
            if let Ok(vec) = s.get_embedding(&content).await {
                let pid = uuid::Uuid::new_v4().to_string();
                let payload = serde_json::json!({
                    "tenant_id": tid, "external_id": eid,
                    "namespace": ns, "agent_id": aid,
                });
                let _ = s.qdrant_upsert(&pid, vec, payload).await;
            }
        });
    }

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

/// POST /v1/memories/propose — propose a memory for core promotion.
async fn propose_memory(
    tenant: TenantContext,
    State(state): State<AppState>,
    Json(body): Json<ProposeRequest>,
) -> Result<(StatusCode, Json<ProposalResponse>), AppError> {
    if body.content.is_empty() {
        return Err(AppError::BadRequest("content must not be empty".into()));
    }
    if body.justification.is_empty() {
        return Err(AppError::BadRequest("justification must not be empty".into()));
    }

    let namespace = body.namespace.clone().unwrap_or_else(|| "default".into());
    let agent_id = body.agent_id.clone().unwrap_or_else(|| "default".into());

    // 1. Create the memory as provisional
    let create_req = CreateMemoryRequest {
        content: body.content.clone(),
        tags: body.tags.clone().unwrap_or_default(),
        namespace: namespace.clone(),
        agent_id: Some(agent_id.clone()),
        metadata: serde_json::json!({}),
        ttl_seconds: None,
        validate: None,
        created_by: Some(agent_id.clone()),
        confidence: body.confidence,
        source: Some("propose".into()),
        memory_type: Some("provisional".into()),
    };
    let mem_row = db::insert_memory(&state.db, tenant.tenant_id, &create_req).await?;

    // Embed the provisional memory
    if !state.config.openai_api_key.is_empty() {
        let s = state.clone();
        let content = mem_row.content.clone();
        let eid = mem_row.external_id.clone();
        let tid = mem_row.tenant_id.to_string();
        let ns = mem_row.namespace.clone();
        let aid = mem_row.agent_id.clone();
        tokio::spawn(async move {
            if let Ok(vec) = s.get_embedding(&content).await {
                let pid = uuid::Uuid::new_v4().to_string();
                let payload = serde_json::json!({
                    "tenant_id": tid, "external_id": eid,
                    "namespace": ns, "agent_id": aid,
                });
                let _ = s.qdrant_upsert(&pid, vec, payload).await;
            }
        });
    }

    // 2. Create proposal record
    let proposal = proposals_db::insert_proposal(
        &state.db,
        tenant.tenant_id,
        mem_row.id,
        &agent_id,
        &body.justification,
        &body.evidence,
        body.confidence,
        None,
    )
    .await?;

    // 3. Fetch evidence contents
    let mut evidence_contents = Vec::new();
    for eid in &body.evidence {
        if let Some(row) = db::get_memory_by_external_id(&state.db, tenant.tenant_id, eid).await? {
            evidence_contents.push(row.content);
        }
    }

    // 4. Fetch top 5 existing core memories via Qdrant similarity
    let mut existing_core_contents = Vec::new();
    if !state.config.openai_api_key.is_empty() {
        if let Ok(vector) = state.get_embedding(&body.content).await {
            let filter = serde_json::json!({
                "must": [
                    {"key": "tenant_id", "match": {"value": tenant.tenant_id.to_string()}},
                    {"key": "namespace", "match": {"value": &namespace}}
                ]
            });
            if let Ok(hits) = state.qdrant_search(vector, 5, 0.3, Some(filter)).await {
                for (ext_id, _score) in &hits {
                    if *ext_id == mem_row.external_id {
                        continue; // skip self
                    }
                    if let Ok(Some(row)) = db::get_memory_by_external_id(&state.db, tenant.tenant_id, ext_id).await {
                        if row.memory_type == "core" {
                            existing_core_contents.push(row.content);
                        }
                    }
                }
            }
        }
    }

    // 5. Build judge prompt and call LLM
    let judge_model = db::get_tenant_judge_model(&state.db, tenant.tenant_id).await?;

    let evidence_str = if evidence_contents.is_empty() {
        "(none provided)".to_string()
    } else {
        evidence_contents.iter().enumerate()
            .map(|(i, c)| format!("{}. {}", i + 1, c))
            .collect::<Vec<_>>().join("\n")
    };
    let existing_str = if existing_core_contents.is_empty() {
        "(none found)".to_string()
    } else {
        existing_core_contents.iter().enumerate()
            .map(|(i, c)| format!("{}. {}", i + 1, c))
            .collect::<Vec<_>>().join("\n")
    };

    let prompt = JUDGE_SYSTEM_PROMPT
        .replace("{content}", &body.content)
        .replace("{justification}", &body.justification)
        .replace("{evidence_contents}", &evidence_str)
        .replace("{existing_core_contents}", &existing_str);

    let judge_response = crate::llm::call_llm(&state.config.openai_api_key, &judge_model, &prompt)
        .await
        .map_err(|e| AppError::Internal(e))?;

    // Parse judge decision
    let json_str = judge_response
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    let judge_decision: serde_json::Value = serde_json::from_str(json_str)
        .unwrap_or_else(|_| serde_json::json!({"decision": "reject", "reason": "Could not parse judge response"}));

    let decision = judge_decision["decision"].as_str().unwrap_or("reject");

    // 6. Act on decision
    let final_status;
    match decision {
        "approve" => {
            final_status = "approved";
            db::update_memory_type(&state.db, tenant.tenant_id, &mem_row.external_id, "core").await?;
            state.ws.broadcast(MemoryEvent {
                event_type: "memory.promoted_to_core".into(),
                memory_id: Some(mem_row.external_id.clone()),
                namespace: Some(namespace.clone()),
                agent_id: Some(agent_id.clone()),
                memory: None,
                tenant_id: tenant.tenant_id,
            });
        }
        "merge" => {
            final_status = "merged";
            let merge_content = judge_decision["suggested_merge_content"]
                .as_str()
                .unwrap_or(&body.content);
            db::update_memory_content_and_type(
                &state.db, tenant.tenant_id, &mem_row.external_id,
                merge_content, "core",
            ).await?;
            state.ws.broadcast(MemoryEvent {
                event_type: "memory.promoted_to_core".into(),
                memory_id: Some(mem_row.external_id.clone()),
                namespace: Some(namespace.clone()),
                agent_id: Some(agent_id.clone()),
                memory: None,
                tenant_id: tenant.tenant_id,
            });
        }
        _ => {
            final_status = "rejected";
            state.ws.broadcast(MemoryEvent {
                event_type: "memory.proposal_rejected".into(),
                memory_id: Some(mem_row.external_id.clone()),
                namespace: Some(namespace.clone()),
                agent_id: Some(agent_id.clone()),
                memory: None,
                tenant_id: tenant.tenant_id,
            });
        }
    }

    // Update proposal with decision
    let updated_proposal = proposals_db::decide_proposal(
        &state.db,
        tenant.tenant_id,
        &proposal.external_id,
        final_status,
        "judge",
        Some(&judge_model),
        Some(&judge_decision),
    )
    .await?
    .unwrap_or(proposal);

    let resp = proposal_to_response(&updated_proposal, &mem_row.external_id);
    Ok((StatusCode::CREATED, Json(resp)))
}

/// GET /v1/memories/proposals — list proposals.
async fn list_proposals_handler(
    tenant: TenantContext,
    State(state): State<AppState>,
    Query(params): Query<ListProposalsQuery>,
) -> Result<Json<Vec<ProposalResponse>>, AppError> {
    let limit = params.limit.min(100) as i64;
    let offset = params.offset as i64;

    let rows = proposals_db::list_proposals(
        &state.db,
        tenant.tenant_id,
        params.status.as_deref(),
        limit,
        offset,
    )
    .await?;

    let mut results = Vec::with_capacity(rows.len());
    for row in &rows {
        let mem_eid = proposals_db::get_memory_external_id(&state.db, row.memory_id)
            .await?
            .unwrap_or_else(|| row.memory_id.to_string());
        results.push(proposal_to_response(row, &mem_eid));
    }

    Ok(Json(results))
}

/// POST /v1/memories/proposals/:id/decide — human override.
async fn decide_proposal_handler(
    tenant: TenantContext,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<DecideRequest>,
) -> Result<Json<ProposalResponse>, AppError> {
    // Validate decision
    if !["approve", "reject", "merge"].contains(&body.decision.as_str()) {
        return Err(AppError::BadRequest("decision must be approve, reject, or merge".into()));
    }

    // Get existing proposal to find memory
    let existing = proposals_db::get_proposal(&state.db, tenant.tenant_id, &id)
        .await?
        .ok_or(AppError::NotFound)?;

    if existing.status != "pending" {
        return Err(AppError::BadRequest(format!("proposal already decided: {}", existing.status)));
    }

    let mem_eid = proposals_db::get_memory_external_id(&state.db, existing.memory_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let decided_by = body.decided_by.clone().unwrap_or_else(|| "human".into());
    let judge_decision = serde_json::json!({
        "decision": body.decision,
        "reason": body.reason.as_deref().unwrap_or("human override"),
    });

    let final_status = match body.decision.as_str() {
        "approve" => {
            db::update_memory_type(&state.db, tenant.tenant_id, &mem_eid, "core").await?;
            state.ws.broadcast(MemoryEvent {
                event_type: "memory.promoted_to_core".into(),
                memory_id: Some(mem_eid.clone()),
                namespace: None,
                agent_id: None,
                memory: None,
                tenant_id: tenant.tenant_id,
            });
            "approved"
        }
        "merge" => {
            let content = body.merged_content.as_deref()
                .ok_or_else(|| AppError::BadRequest("merged_content required for merge decision".into()))?;
            db::update_memory_content_and_type(&state.db, tenant.tenant_id, &mem_eid, content, "core").await?;
            state.ws.broadcast(MemoryEvent {
                event_type: "memory.promoted_to_core".into(),
                memory_id: Some(mem_eid.clone()),
                namespace: None,
                agent_id: None,
                memory: None,
                tenant_id: tenant.tenant_id,
            });
            "merged"
        }
        _ => {
            state.ws.broadcast(MemoryEvent {
                event_type: "memory.proposal_rejected".into(),
                memory_id: Some(mem_eid.clone()),
                namespace: None,
                agent_id: None,
                memory: None,
                tenant_id: tenant.tenant_id,
            });
            "rejected"
        }
    };

    let updated = proposals_db::decide_proposal(
        &state.db,
        tenant.tenant_id,
        &id,
        final_status,
        &decided_by,
        None,
        Some(&judge_decision),
    )
    .await?
    .ok_or_else(|| AppError::BadRequest("proposal could not be updated".into()))?;

    Ok(Json(proposal_to_response(&updated, &mem_eid)))
}

fn proposal_to_response(row: &proposals_db::ProposalRow, memory_external_id: &str) -> ProposalResponse {
    ProposalResponse {
        id: row.external_id.clone(),
        memory_id: memory_external_id.to_string(),
        proposed_by: row.proposed_by.clone(),
        justification: row.justification.clone(),
        evidence_ids: row.evidence_ids.clone(),
        confidence: row.confidence.unwrap_or(0.8),
        status: row.status.clone(),
        judge_model: row.judge_model.clone(),
        judge_decision: row.judge_decision.clone(),
        decided_at: row.decided_at,
        decided_by: row.decided_by.clone(),
        created_at: row.created_at,
        expires_at: row.expires_at,
    }
}

pub fn proposal_routes() -> Router<AppState> {
    Router::new()
        .route("/v1/memories/private", post(create_private_memory))
        .route("/v1/memories/propose", post(propose_memory))
        .route("/v1/memories/proposals", get(list_proposals_handler))
        .route("/v1/memories/proposals/{id}/decide", post(decide_proposal_handler))
}
