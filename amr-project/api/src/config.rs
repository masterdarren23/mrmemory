use std::env;
use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct Config {
    pub listen_addr: SocketAddr,
    pub database_url: String,
    pub qdrant_url: String,
    pub redis_url: String,
    pub openai_api_key: String,
    pub embedding_model: String,
    pub pg_max_connections: u32,
    /// Rate limit: requests per minute for starter plan
    pub rate_limit_starter: u32,
    /// Rate limit: requests per minute for pro plan
    pub rate_limit_pro: u32,
    /// Stripe webhook signing secret
    pub stripe_webhook_secret: String,
    /// Stripe secret key for API calls
    pub stripe_secret_key: String,
    /// Max memories per namespace for starter plan
    pub max_memories_starter: i64,
    /// Max memories per namespace for pro plan
    pub max_memories_pro: i64,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            listen_addr: env::var("LISTEN_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:8080".into())
                .parse()
                .expect("invalid LISTEN_ADDR"),
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL is required"),
            qdrant_url: env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6334".into()),
            redis_url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".into()),
            openai_api_key: env::var("OPENAI_API_KEY").unwrap_or_default(),
            embedding_model: env::var("EMBEDDING_MODEL")
                .unwrap_or_else(|_| "text-embedding-3-small".into()),
            pg_max_connections: env::var("PG_MAX_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            rate_limit_starter: env::var("RATE_LIMIT_STARTER")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
            rate_limit_pro: env::var("RATE_LIMIT_PRO")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1000),
            stripe_webhook_secret: env::var("STRIPE_WEBHOOK_SECRET").unwrap_or_default(),
            stripe_secret_key: env::var("STRIPE_SECRET_KEY").unwrap_or_default(),
            max_memories_starter: env::var("MAX_MEMORIES_STARTER")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10_000),
            max_memories_pro: env::var("MAX_MEMORIES_PRO")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100_000),
        }
    }
}
