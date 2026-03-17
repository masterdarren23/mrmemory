use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};

async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn health_ready() -> Json<Value> {
    // TODO: check PG, Qdrant, Redis connectivity
    Json(json!({
        "status": "ready",
        "postgres": "stub",
        "qdrant": "stub",
        "redis": "stub"
    }))
}

pub fn health_routes() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/health/ready", get(health_ready))
}
