pub mod keys;

use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

use self::keys::hash_api_key;
use crate::state::AppState;

#[derive(Debug, Clone)]
pub struct TenantContext {
    pub tenant_id: Uuid,
    pub key_id: Uuid,
    pub plan: String,
    pub scopes: Vec<String>,
}

pub struct AuthRejection(&'static str);

impl IntoResponse for AuthRejection {
    fn into_response(self) -> Response {
        let body = json!({
            "error": {
                "code": "unauthorized",
                "message": self.0
            }
        });
        (StatusCode::UNAUTHORIZED, Json(body)).into_response()
    }
}

#[derive(Debug, sqlx::FromRow)]
struct KeyRow {
    key_id: Uuid,
    tenant_id: Uuid,
    plan: String,
    scopes: Vec<String>,
}

// Manually implement the desugared #[async_trait] version of FromRequestParts
impl axum::extract::FromRequestParts<AppState> for TenantContext {
    type Rejection = AuthRejection;

    fn from_request_parts<'life0, 'life1, 'async_trait>(
        parts: &'life0 mut Parts,
        state: &'life1 AppState,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Rejection>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let auth_header = parts
                .headers
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .ok_or(AuthRejection("missing Authorization header"))?;

            let token = auth_header
                .strip_prefix("Bearer ")
                .ok_or(AuthRejection("expected Bearer token"))?;

            if !token.starts_with("amr_sk_") {
                return Err(AuthRejection("invalid key format"));
            }

            let key_hash = hash_api_key(token);

            let row: KeyRow = sqlx::query_as(
                "SELECT ak.id as key_id, ak.tenant_id, t.plan, ak.scopes FROM api_keys ak JOIN tenants t ON t.id = ak.tenant_id WHERE ak.key_hash = $1 AND ak.revoked_at IS NULL AND (ak.expires_at IS NULL OR ak.expires_at > now())"
            )
            .bind(&key_hash)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| {
                tracing::error!("auth query error: {}", e);
                AuthRejection("internal error")
            })?
            .ok_or(AuthRejection("invalid API key"))?;

            let db = state.db.clone();
            let key_id = row.key_id;
            tokio::spawn(async move {
                let _ = sqlx::query("UPDATE api_keys SET last_used_at = now() WHERE id = $1")
                    .bind(key_id)
                    .execute(&db)
                    .await;
            });

            Ok(TenantContext {
                tenant_id: row.tenant_id,
                key_id: row.key_id,
                plan: row.plan,
                scopes: row.scopes,
            })
        })
    }
}
