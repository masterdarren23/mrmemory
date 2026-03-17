pub mod keys;

use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use uuid::Uuid;

use self::keys::hash_api_key;
use crate::state::AppState;

/// Extracted from a valid API key. Available in route handlers via extractor.
#[derive(Debug, Clone)]
pub struct TenantContext {
    pub tenant_id: Uuid,
    pub key_id: Uuid,
    pub plan: String,
    pub scopes: Vec<String>,
}

/// Rejection when auth fails.
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

/// Row returned from the api_keys + tenants join.
#[derive(Debug)]
struct KeyRow {
    key_id: Uuid,
    tenant_id: Uuid,
    plan: String,
    scopes: Vec<String>,
}

impl FromRequestParts<AppState> for TenantContext {
    type Rejection = AuthRejection;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
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

        // Look up key in database
        let row = sqlx::query_as!(
            KeyRow,
            r#"
            SELECT
                ak.id as key_id,
                ak.tenant_id,
                t.plan,
                ak.scopes
            FROM api_keys ak
            JOIN tenants t ON t.id = ak.tenant_id
            WHERE ak.key_hash = $1
              AND ak.revoked_at IS NULL
              AND (ak.expires_at IS NULL OR ak.expires_at > now())
            "#,
            &key_hash
        )
        .fetch_optional(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("auth query error: {}", e);
            AuthRejection("internal error")
        })?
        .ok_or(AuthRejection("invalid API key"))?;

        // Update last_used_at (fire-and-forget, don't block auth)
        let db = state.db.clone();
        let key_id = row.key_id;
        tokio::spawn(async move {
            let _ = sqlx::query!(
                "UPDATE api_keys SET last_used_at = now() WHERE id = $1",
                key_id
            )
            .execute(&db)
            .await;
        });

        Ok(TenantContext {
            tenant_id: row.tenant_id,
            key_id: row.key_id,
            plan: row.plan,
            scopes: row.scopes,
        })
    }
}
