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
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            listen_addr: env::var("LISTEN_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:8080".into())
                .parse()
                .expect("invalid LISTEN_ADDR"),
            database_url: env::var("DATABASE_URL")
                .expect("DATABASE_URL is required"),
            qdrant_url: env::var("QDRANT_URL")
                .unwrap_or_else(|_| "http://localhost:6334".into()),
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".into()),
            openai_api_key: env::var("OPENAI_API_KEY").unwrap_or_default(),
            embedding_model: env::var("EMBEDDING_MODEL")
                .unwrap_or_else(|_| "text-embedding-ada-002".into()),
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
        }
    }
}
