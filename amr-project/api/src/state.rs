use crate::config::Config;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;

/// Shared application state. Cloneable (all fields are Arc or Clone).
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Arc<Config>,
}

impl AppState {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let db = PgPoolOptions::new()
            .max_connections(config.pg_max_connections)
            .connect(&config.database_url)
            .await?;

        tracing::info!("Connected to PostgreSQL");

        // Run migrations
        sqlx::migrate!("./migrations").run(&db).await?;
        tracing::info!("Migrations applied");

        Ok(Self {
            db,
            config: Arc::new(config),
        })
    }
}
