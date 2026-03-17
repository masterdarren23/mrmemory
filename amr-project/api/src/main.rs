mod auth;
mod config;
mod db;
mod error;
mod models;
mod routes;
mod state;

use crate::config::Config;
use crate::routes::{create_key, health_routes, memory_routes, stripe_webhook, welcome_page};
use crate::state::AppState;

use axum::routing::{get, post};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config = Config::from_env();
    let addr = config.listen_addr;
    let state = AppState::new(config).await.expect("failed to init state");

    let app = memory_routes()
        .route("/v1/auth/keys", post(create_key))
        .route("/v1/billing/webhook", post(stripe_webhook))
        .route("/v1/welcome", get(welcome_page))
        .with_state(state)
        .merge(health_routes())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    tracing::info!("AMR server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
