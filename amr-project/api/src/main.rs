mod auth;
mod config;
mod db;
mod error;
pub mod llm;
mod models;
mod routes;
mod state;
pub mod ws;

use crate::config::Config;
use crate::routes::{
    auto_remember, compress_memories, create_key, health_routes, memory_routes, stats_routes,
    stripe_webhook, welcome_page, ws_handler,
};
use crate::state::AppState;

use axum::http::{header, Method};
use axum::routing::{get, post};
use tower_http::cors::{AllowOrigin, CorsLayer};
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

    // Spawn background TTL pruning task.
    {
        let prune_db = state.db.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                match crate::db::memories::prune_expired_memories(&prune_db).await {
                    Ok(pruned) if !pruned.is_empty() => {
                        tracing::info!("TTL pruner: removed {} expired memories", pruned.len());
                    }
                    Ok(_) => {}
                    Err(e) => {
                        tracing::warn!("TTL pruner error: {}", e);
                    }
                }
            }
        });
        tracing::info!("Background TTL pruning task started (60s interval)");
    }

    let app = memory_routes()
        .merge(health_routes())
        .merge(stats_routes())
        .route("/v1/memories/auto", post(auto_remember))
        .route("/v1/memories/compress", post(compress_memories))
        .route("/v1/ws", get(ws_handler))
        .route("/v1/auth/keys", post(create_key))
        .route("/v1/billing/webhook", post(stripe_webhook))
        .route("/v1/welcome", get(welcome_page))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(AllowOrigin::list([
                    "https://mrmemory.dev".parse().unwrap(),
                    "https://www.mrmemory.dev".parse().unwrap(),
                    "http://localhost:3000".parse().unwrap(),
                    "http://localhost:8080".parse().unwrap(),
                ]))
                .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE, Method::OPTIONS])
                .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
                .max_age(std::time::Duration::from_secs(3600)),
        );

    eprintln!("AMR: routes configured, binding to {}...", addr);
    tracing::info!("AMR server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
