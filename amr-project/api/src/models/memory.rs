use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Stored memory record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: Uuid,
    pub external_id: String,
    pub tenant_id: Uuid,
    pub agent_id: String,
    pub namespace: String,
    pub content: String,
    pub tags: Vec<String>,
    pub metadata: serde_json::Value,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// POST /v1/memories request body.
#[derive(Debug, Deserialize)]
pub struct CreateMemoryRequest {
    pub content: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_namespace")]
    pub namespace: String,
    pub agent_id: Option<String>,
    #[serde(default)]
    pub metadata: serde_json::Value,
    pub ttl_seconds: Option<i64>,
}

fn default_namespace() -> String {
    "default".into()
}

/// Response for a single memory.
#[derive(Debug, Serialize)]
pub struct MemoryResponse {
    pub id: String,
    pub content: String,
    pub tags: Vec<String>,
    pub namespace: String,
    pub agent_id: String,
    pub metadata: serde_json::Value,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// GET /v1/memories/recall query params.
#[derive(Debug, Deserialize)]
pub struct RecallQuery {
    pub query: String,
    pub namespace: Option<String>,
    pub agent_id: Option<String>,
    pub tags: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default = "default_threshold")]
    pub threshold: f32,
}

fn default_limit() -> usize {
    10
}
fn default_threshold() -> f32 {
    0.5
}

/// Single recall result.
#[derive(Debug, Serialize)]
pub struct RecallResult {
    pub id: String,
    pub content: String,
    pub tags: Vec<String>,
    pub namespace: String,
    pub agent_id: String,
    pub metadata: serde_json::Value,
    pub similarity: f32,
    pub created_at: DateTime<Utc>,
}

/// GET /v1/memories/recall response.
#[derive(Debug, Serialize)]
pub struct RecallResponse {
    pub memories: Vec<RecallResult>,
    pub count: usize,
    pub query_time_ms: u64,
}

/// GET /v1/memories query params.
#[derive(Debug, Deserialize)]
pub struct ListMemoriesQuery {
    pub namespace: Option<String>,
    pub agent_id: Option<String>,
    pub tags: Option<String>,
    #[serde(default = "default_list_limit")]
    pub limit: usize,
    #[serde(default)]
    pub offset: usize,
}

fn default_list_limit() -> usize {
    20
}

/// GET /v1/memories response.
#[derive(Debug, Serialize)]
pub struct ListMemoriesResponse {
    pub memories: Vec<MemoryResponse>,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
}

impl Memory {
    pub fn to_response(&self) -> MemoryResponse {
        MemoryResponse {
            id: self.external_id.clone(),
            content: self.content.clone(),
            tags: self.tags.clone(),
            namespace: self.namespace.clone(),
            agent_id: self.agent_id.clone(),
            metadata: self.metadata.clone(),
            expires_at: self.expires_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}
