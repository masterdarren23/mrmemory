mod auth;
mod config;
mod db;
mod error;
mod models;
mod routes;

use crate::config::Config;
use crate::db::MemoryStore;
use axum::routing::post;
use crate::routes::{create_key, health_routes, memory_routes};

use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    // Init tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config = Config::from_env();
    let addr = config.listen_addr;
    let store = MemoryStore::new();

    let app = memory_routes()
        .with_state(store)
        .route("/v1/auth/keys", post(create_key))
        .merge(health_routes())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    tracing::info!("AMR server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
