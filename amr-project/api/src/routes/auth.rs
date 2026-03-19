use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::auth::keys::key_prefix;
use crate::auth::TenantContext;
use crate::db::keys::create_api_key;
use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateKeyRequest {
    pub name: String,
    #[serde(default = "default_scopes")]
    pub scopes: Vec<String>,
}

fn default_scopes() -> Vec<String> {
    vec![
        "memories:read".into(),
        "memories:write".into(),
        "memories:share".into(),
        "memories:delete".into(),
        "usage:read".into(),
    ]
}

#[derive(Debug, Serialize)]
pub struct CreateKeyResponse {
    pub id: String,
    pub name: String,
    pub key: String,
    pub prefix: String,
    pub scopes: Vec<String>,
    pub created_at: String,
}

/// POST /v1/auth/keys — generate a new API key.
pub async fn create_key(
    tenant: TenantContext,
    State(state): State<AppState>,
    Json(body): Json<CreateKeyRequest>,
) -> Result<(StatusCode, Json<CreateKeyResponse>), AppError> {
    if body.name.is_empty() || body.name.len() > 128 {
        return Err(AppError::BadRequest("name must be 1-128 characters".into()));
    }

    let (external_id, raw_key) =
        create_api_key(&state.db, tenant.tenant_id, &body.name, &body.scopes).await?;

    let response = CreateKeyResponse {
        id: external_id,
        name: body.name,
        prefix: key_prefix(&raw_key),
        key: raw_key,
        scopes: body.scopes,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}
