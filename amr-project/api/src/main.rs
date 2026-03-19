mod auth;
mod config;
mod db;
mod error;
mod models;
mod routes;
mod state;
pub mod ws;

use crate::config::Config;
use crate::routes::{
    create_key, health_routes, memory_routes, stripe_webhook, welcome_page, ws_handler,
};
use crate::state::AppState;

use axum::routing::{get, post};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    eprintln!("AMR: starting up...");

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    eprintln!("AMR: tracing initialized, loading config...");
    let config = Config::from_env();
    eprintln!(
        "AMR: config loaded, connecting to DB at {}...",
        &config.database_url[..20]
    );
    let addr = config.listen_addr;

    let state = match AppState::new(config).await {
        Ok(s) => {
            eprintln!("AMR: state initialized successfully");
            s
        }
        Err(e) => {
            eprintln!("AMR: FATAL - failed to init state: {}", e);
            std::process::exit(1);
        }
    };

    let app = memory_routes()
        .merge(health_routes())
        .route("/v1/ws", get(ws_handler))
        .route("/v1/auth/keys", post(create_key))
        .route("/v1/billing/webhook", post(stripe_webhook))
        .route("/v1/welcome", get(welcome_page))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    eprintln!("AMR: routes configured, binding to {}...", addr);
    tracing::info!("AMR server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
