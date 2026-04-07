use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};

use crate::state::AppState;

async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn health_ready(State(state): State<AppState>) -> Json<Value> {
    // Check PostgreSQL
    let pg_status = match sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.db)
        .await
    {
        Ok(_) => "ok",
        Err(_) => "error",
    };

    // Check Qdrant
    let qdrant_status = match state
        .http
        .get(&format!("{}/collections", state.config.qdrant_url))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => "ok",
        _ => "error",
    };

    let overall = if pg_status == "ok" && qdrant_status == "ok" {
        "ready"
    } else {
        "degraded"
    };

    Json(json!({
        "status": overall,
        "postgres": pg_status,
        "qdrant": qdrant_status,
    }))
}

pub fn health_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/health/ready", get(health_ready))
}
