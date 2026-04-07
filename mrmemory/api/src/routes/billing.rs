use axum::body::Bytes;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::Html;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use uuid::Uuid;

use crate::db::keys::create_api_key;
use crate::error::AppError;
use crate::state::AppState;

type HmacSha256 = Hmac<Sha256>;

// ── Stripe Webhook ──────────────────────────────────────────

pub async fn stripe_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, AppError> {
    let sig_header = headers
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::BadRequest("missing stripe-signature".into()))?;

    verify_stripe_signature(&body, sig_header, &state.config.stripe_webhook_secret)?;

    let event: serde_json::Value =
        serde_json::from_slice(&body).map_err(|e| AppError::BadRequest(e.to_string()))?;

    let event_type = event["type"].as_str().unwrap_or("");
    tracing::info!("Stripe event: {}", event_type);

    if event_type == "checkout.session.completed" {
        handle_checkout_completed(&state, &event["data"]["object"]).await?;
    }

    Ok(StatusCode::OK)
}

async fn handle_checkout_completed(
    state: &AppState,
    session: &serde_json::Value,
) -> Result<(), AppError> {
    let email = session["customer_details"]["email"]
        .as_str()
        .or_else(|| session["customer_email"].as_str())
        .ok_or_else(|| AppError::BadRequest("no email in session".into()))?;

    let stripe_customer_id = session["customer"].as_str().unwrap_or("");
    let stripe_session_id = session["id"].as_str().unwrap_or("");

    let name = email.split('@').next().unwrap_or("user");
    let external_id = format!("tenant_{}", &Uuid::new_v4().to_string()[..12]);

    // Check if tenant already exists (idempotency)
    let existing: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM tenants WHERE email = $1")
        .bind(email)
        .fetch_optional(&state.db)
        .await?;

    if existing.is_some() {
        tracing::info!("Tenant already exists for {}, skipping", email);
        sqlx::query(
            "UPDATE tenants SET stripe_customer_id = COALESCE(stripe_customer_id, $1), stripe_session_id = COALESCE(stripe_session_id, $2) WHERE email = $3"
        )
        .bind(stripe_customer_id)
        .bind(stripe_session_id)
        .bind(email)
        .execute(&state.db)
        .await?;
        return Ok(());
    }

    // Create tenant
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO tenants (external_id, name, email, plan, stripe_customer_id, stripe_session_id) VALUES ($1, $2, $3, 'starter', $4, $5) RETURNING id"
    )
    .bind(&external_id)
    .bind(name)
    .bind(email)
    .bind(stripe_customer_id)
    .bind(stripe_session_id)
    .fetch_one(&state.db)
    .await?;

    let tenant_id = row.0;

    let scopes = vec![
        "memories:read".into(),
        "memories:write".into(),
        "memories:share".into(),
        "memories:delete".into(),
        "usage:read".into(),
    ];
    let (_key_id, _raw_key) = create_api_key(&state.db, tenant_id, "default", &scopes).await?;

    tracing::info!("Created tenant {} with API key for {}", external_id, email);
    Ok(())
}

fn verify_stripe_signature(payload: &[u8], sig_header: &str, secret: &str) -> Result<(), AppError> {
    let mut timestamp = None;
    let mut signature = None;

    for part in sig_header.split(',') {
        let mut kv = part.splitn(2, '=');
        match (kv.next(), kv.next()) {
            (Some("t"), Some(v)) => timestamp = Some(v.to_string()),
            (Some("v1"), Some(v)) => signature = Some(v.to_string()),
            _ => {}
        }
    }

    let timestamp = timestamp.ok_or_else(|| AppError::BadRequest("missing timestamp".into()))?;
    let signature = signature.ok_or_else(|| AppError::BadRequest("missing signature".into()))?;

    let signed_payload = format!("{}.{}", timestamp, String::from_utf8_lossy(payload));
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|_| AppError::BadRequest("bad secret".into()))?;
    mac.update(signed_payload.as_bytes());
    let expected = hex::encode(mac.finalize().into_bytes());

    if !constant_time_eq(expected.as_bytes(), signature.as_bytes()) {
        return Err(AppError::Unauthorized);
    }

    Ok(())
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

// ── Welcome Page ────────────────────────────────────────────

#[derive(serde::Deserialize)]
pub struct WelcomeQuery {
    pub session_id: Option<String>,
}

pub async fn welcome_page(
    State(state): State<AppState>,
    Query(params): Query<WelcomeQuery>,
) -> Html<String> {
    let session_id = match &params.session_id {
        Some(s) if !s.is_empty() => s.as_str(),
        _ => return Html(generic_welcome()),
    };

    let result = lookup_by_session(&state, session_id).await;

    match result {
        Ok(Some(info)) => Html(personalized_welcome(&info)),
        _ => match fetch_session_and_provision(&state, session_id).await {
            Ok(Some(info)) => Html(personalized_welcome(&info)),
            _ => Html(generic_welcome()),
        },
    }
}

struct WelcomeInfo {
    email: String,
    api_key: String,
}

async fn lookup_by_session(
    state: &AppState,
    session_id: &str,
) -> Result<Option<WelcomeInfo>, AppError> {
    let tenant: Option<(Uuid, String)> =
        sqlx::query_as("SELECT id, email FROM tenants WHERE stripe_session_id = $1")
            .bind(session_id)
            .fetch_optional(&state.db)
            .await?;

    let (tenant_id, email) = match tenant {
        Some(t) => t,
        None => return Ok(None),
    };

    let scopes = vec![
        "memories:read".into(),
        "memories:write".into(),
        "memories:share".into(),
        "memories:delete".into(),
        "usage:read".into(),
    ];
    let (_key_id, raw_key) = create_api_key(&state.db, tenant_id, "welcome-page", &scopes).await?;

    Ok(Some(WelcomeInfo {
        email,
        api_key: raw_key,
    }))
}

async fn fetch_session_and_provision(
    state: &AppState,
    session_id: &str,
) -> Result<Option<WelcomeInfo>, AppError> {
    if state.config.stripe_secret_key.is_empty() {
        return Ok(None);
    }

    let client = reqwest::Client::new();
    let resp = client
        .get(&format!(
            "https://api.stripe.com/v1/checkout/sessions/{}",
            session_id
        ))
        .bearer_auth(&state.config.stripe_secret_key)
        .send()
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    if !resp.status().is_success() {
        tracing::warn!("Stripe session fetch failed: {}", resp.status());
        return Ok(None);
    }

    let session: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    let email = session["customer_details"]["email"]
        .as_str()
        .or_else(|| session["customer_email"].as_str());

    let email = match email {
        Some(e) => e,
        None => return Ok(None),
    };

    let stripe_customer_id = session["customer"].as_str().unwrap_or("");

    let existing: Option<(Uuid, String)> =
        sqlx::query_as("SELECT id, email FROM tenants WHERE email = $1")
            .bind(email)
            .fetch_optional(&state.db)
            .await?;

    let tenant_id = if let Some((id, _)) = existing {
        sqlx::query(
            "UPDATE tenants SET stripe_session_id = $1, stripe_customer_id = COALESCE(stripe_customer_id, $2) WHERE id = $3"
        )
        .bind(session_id)
        .bind(stripe_customer_id)
        .bind(id)
        .execute(&state.db)
        .await?;
        id
    } else {
        let external_id = format!("tenant_{}", &Uuid::new_v4().to_string()[..12]);
        let name = email.split('@').next().unwrap_or("user");
        let row: (Uuid,) = sqlx::query_as(
            "INSERT INTO tenants (external_id, name, email, plan, stripe_customer_id, stripe_session_id) VALUES ($1, $2, $3, 'starter', $4, $5) RETURNING id"
        )
        .bind(&external_id)
        .bind(name)
        .bind(email)
        .bind(stripe_customer_id)
        .bind(session_id)
        .fetch_one(&state.db)
        .await?;
        row.0
    };

    let scopes = vec![
        "memories:read".into(),
        "memories:write".into(),
        "memories:share".into(),
        "memories:delete".into(),
        "usage:read".into(),
    ];
    let (_key_id, raw_key) = create_api_key(&state.db, tenant_id, "welcome-page", &scopes).await?;

    Ok(Some(WelcomeInfo {
        email: email.to_string(),
        api_key: raw_key,
    }))
}

fn personalized_welcome(info: &WelcomeInfo) -> String {
    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Welcome to AMR</title>
<style>
  * {{ margin: 0; padding: 0; box-sizing: border-box; }}
  body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; background: #0a0a0f; color: #e0e0e0; min-height: 100vh; display: flex; align-items: center; justify-content: center; }}
  .card {{ max-width: 640px; width: 90%; background: #141420; border: 1px solid #2a2a3a; border-radius: 16px; padding: 48px; }}
  h1 {{ color: #7c6fe0; font-size: 28px; margin-bottom: 8px; }}
  .subtitle {{ color: #888; margin-bottom: 32px; }}
  .key-box {{ background: #1a1a2e; border: 1px solid #333; border-radius: 8px; padding: 16px; margin: 24px 0; font-family: monospace; font-size: 14px; word-break: break-all; position: relative; }}
  .key-box .label {{ color: #7c6fe0; font-size: 12px; text-transform: uppercase; margin-bottom: 8px; display: block; }}
  .warning {{ color: #f59e0b; font-size: 13px; margin-top: 8px; }}
  pre {{ background: #1a1a2e; border: 1px solid #333; border-radius: 8px; padding: 16px; overflow-x: auto; margin: 16px 0; font-size: 13px; line-height: 1.5; }}
  code {{ color: #a5f3fc; }}
  h2 {{ color: #ccc; font-size: 18px; margin-top: 32px; margin-bottom: 12px; }}
  .copy-btn {{ background: #7c6fe0; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer; font-size: 12px; float: right; }}
  .copy-btn:hover {{ background: #6a5cc8; }}
  a {{ color: #7c6fe0; }}
</style>
</head>
<body>
<div class="card">
  <h1>🎉 Welcome to AMR!</h1>
  <p class="subtitle">Your account is ready, {email}</p>

  <div class="key-box">
    <span class="label">Your API Key</span>
    <button class="copy-btn" onclick="navigator.clipboard.writeText('{api_key}')">Copy</button>
    <code>{api_key}</code>
  </div>
  <p class="warning">⚠️ Save this key now — you won't be able to see it again.</p>

  <h2>Quick Start</h2>
  <pre><code>pip install mrmemory

from mrmemory import AMR
amr = AMR("{api_key}")
amr.remember("User prefers dark mode")
memories = amr.recall("What does the user like?")</code></pre>

  <h2>Next Steps</h2>
  <ul style="margin-left: 20px; line-height: 2;">
    <li>Read the <a href="https://mrmemory.dev/docs">API docs</a></li>
    <li>Python: <code>pip install mrmemory</code></li>
    <li>TypeScript: <code>npm install memorymr</code></li>
  </ul>
</div>
</body>
</html>"##,
        email = info.email,
        api_key = info.api_key,
    )
}

fn generic_welcome() -> String {
    r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Welcome to AMR</title>
<style>
  * { margin: 0; padding: 0; box-sizing: border-box; }
  body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; background: #0a0a0f; color: #e0e0e0; min-height: 100vh; display: flex; align-items: center; justify-content: center; }
  .card { max-width: 640px; width: 90%; background: #141420; border: 1px solid #2a2a3a; border-radius: 16px; padding: 48px; text-align: center; }
  h1 { color: #7c6fe0; font-size: 28px; margin-bottom: 16px; }
  p { color: #888; line-height: 1.6; }
  a { color: #7c6fe0; }
</style>
</head>
<body>
<div class="card">
  <h1>🎉 Thank you!</h1>
  <p>Your AMR account is being set up. Please wait a moment and refresh this page.</p>
  <p style="margin-top: 24px;">If your API key doesn't appear, email <a href="mailto:support@mrmemory.dev">support@mrmemory.dev</a></p>
</div>
</body>
</html>"##
        .to_string()
}
