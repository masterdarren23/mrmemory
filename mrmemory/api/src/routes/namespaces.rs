use axum::extract::{Path, State};
use axum::routing::{get, put};
use axum::Json;
use axum::Router;

use crate::auth::TenantContext;
use crate::db::namespaces;
use crate::error::AppError;
use crate::models::{NamespacePolicyResponse, SetNamespacePolicyRequest};
use crate::state::AppState;

/// PUT /v1/namespaces/:name/policy
async fn set_policy(
    tenant: TenantContext,
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(body): Json<SetNamespacePolicyRequest>,
) -> Result<Json<NamespacePolicyResponse>, AppError> {
    let valid = ["open", "append_only", "read_only"];
    if !valid.contains(&body.policy.as_str()) {
        return Err(AppError::BadRequest(format!(
            "invalid policy '{}', must be one of: open, append_only, read_only",
            body.policy
        )));
    }

    namespaces::set_namespace_policy(&state.db, tenant.tenant_id, &name, &body.policy).await?;

    Ok(Json(NamespacePolicyResponse {
        namespace: name,
        policy: body.policy,
    }))
}

/// GET /v1/namespaces/:name/policy
async fn get_policy(
    tenant: TenantContext,
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<NamespacePolicyResponse>, AppError> {
    let policy = namespaces::get_namespace_policy(&state.db, tenant.tenant_id, &name)
        .await?
        .unwrap_or_else(|| "open".into());

    Ok(Json(NamespacePolicyResponse {
        namespace: name,
        policy,
    }))
}

pub fn namespace_routes() -> Router<AppState> {
    Router::new().route(
        "/v1/namespaces/:name/policy",
        put(set_policy).get(get_policy),
    )
}
