use crate::config::Config;
use crate::ws::WsManager;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;

/// Shared application state. Cloneable (all fields are Arc or Clone).
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Arc<Config>,
    pub ws: WsManager,
    pub http: reqwest::Client,
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

        let http = reqwest::Client::new();

        // Ensure Qdrant "memories" collection exists.
        let qdrant_url = &config.qdrant_url;
        let col_url = format!("{}/collections/memories", qdrant_url);
        let exists = http.get(&col_url).send().await;
        if let Ok(resp) = exists {
            if resp.status() == reqwest::StatusCode::NOT_FOUND || !resp.status().is_success() {
                tracing::info!("Creating Qdrant 'memories' collection");
                let body = serde_json::json!({
                    "vectors": {
                        "size": 1536,
                        "distance": "Cosine"
                    }
                });
                let create_resp = http
                    .put(&col_url)
                    .json(&body)
                    .send()
                    .await;
                match create_resp {
                    Ok(r) if r.status().is_success() => {
                        tracing::info!("Qdrant 'memories' collection created");
                    }
                    Ok(r) => {
                        tracing::warn!("Qdrant collection create response: {}", r.status());
                    }
                    Err(e) => {
                        tracing::warn!("Failed to create Qdrant collection: {}", e);
                    }
                }
            } else {
                tracing::info!("Qdrant 'memories' collection already exists");
            }
        } else {
            tracing::warn!("Could not reach Qdrant at {}, vector search may fail", qdrant_url);
        }

        Ok(Self {
            db,
            config: Arc::new(config),
            ws: WsManager::new(1024),
            http,
        })
    }

    /// Generate an embedding via OpenAI API.
    pub async fn get_embedding(&self, text: &str) -> anyhow::Result<Vec<f32>> {
        let body = serde_json::json!({
            "model": self.config.embedding_model,
            "input": text
        });

        let resp = self
            .http
            .post("https://api.openai.com/v1/embeddings")
            .bearer_auth(&self.config.openai_api_key)
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("OpenAI embeddings API error {}: {}", status, text);
        }

        let json: serde_json::Value = resp.json().await?;
        let embedding = json["data"][0]["embedding"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("unexpected embedding response format"))?
            .iter()
            .filter_map(|v| v.as_f64().map(|f| f as f32))
            .collect();

        Ok(embedding)
    }

    /// Upsert a vector into Qdrant.
    pub async fn qdrant_upsert(&self, point_id: &str, vector: Vec<f32>, payload: serde_json::Value) -> anyhow::Result<()> {
        let body = serde_json::json!({
            "points": [{
                "id": point_id,
                "vector": vector,
                "payload": payload
            }]
        });

        let resp = self
            .http
            .put(&format!("{}/collections/memories/points", self.config.qdrant_url))
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Qdrant upsert error {}: {}", status, text);
        }

        Ok(())
    }

    /// Search Qdrant for similar vectors.
    pub async fn qdrant_search(
        &self,
        vector: Vec<f32>,
        limit: usize,
        threshold: f32,
        filter: Option<serde_json::Value>,
    ) -> anyhow::Result<Vec<(String, f32)>> {
        let mut body = serde_json::json!({
            "vector": vector,
            "limit": limit,
            "score_threshold": threshold,
            "with_payload": true
        });

        if let Some(f) = filter {
            body["filter"] = f;
        }

        let resp = self
            .http
            .post(&format!("{}/collections/memories/points/search", self.config.qdrant_url))
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Qdrant search error {}: {}", status, text);
        }

        let json: serde_json::Value = resp.json().await?;
        let results = json["result"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("unexpected Qdrant search response"))?
            .iter()
            .filter_map(|r| {
                let external_id = r["payload"]["external_id"].as_str()?.to_string();
                let score = r["score"].as_f64()? as f32;
                Some((external_id, score))
            })
            .collect();

        Ok(results)
    }
}
