use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::keys::{generate_api_key, hash_api_key, key_prefix};

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
/// In production this requires JWT (dashboard) auth, not API key auth.
/// For now it's open for development.
pub async fn create_key(
    Json(body): Json<CreateKeyRequest>,
) -> Result<(StatusCode, Json<CreateKeyResponse>), StatusCode> {
    if body.name.is_empty() || body.name.len() > 128 {
        return Err(StatusCode::BAD_REQUEST);
    }

    let raw_key = generate_api_key();
    let _hash = hash_api_key(&raw_key);
    let prefix = key_prefix(&raw_key);
    let key_id = Uuid::new_v4();
    let external_id = format!("key_{}", &key_id.to_string()[..12]);

    // STUB: In production, insert into api_keys table:
    //   INSERT INTO api_keys (id, external_id, tenant_id, name, key_hash, key_prefix, scopes)
    //   VALUES ($1, $2, $3, $4, $5, $6, $7)
    // TODO: wire to PG

    let response = CreateKeyResponse {
        id: external_id,
        name: body.name,
        key: raw_key,
        prefix,
        scopes: body.scopes,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}
