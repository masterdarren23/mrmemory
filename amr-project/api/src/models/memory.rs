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

/// PATCH /v1/memories/:id request body.
#[derive(Debug, Deserialize)]
pub struct UpdateMemoryRequest {
    pub content: Option<String>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
}

/// DELETE /v1/memories/outdated request body.
#[derive(Debug, Deserialize)]
pub struct DeleteOutdatedRequest {
    /// Delete memories older than this many seconds.
    pub older_than_seconds: Option<i64>,
    /// Only delete memories with ALL of these tags.
    pub tags: Option<Vec<String>>,
    /// Scope to namespace.
    pub namespace: Option<String>,
    /// Scope to agent.
    pub agent_id: Option<String>,
    /// Dry run — return count without deleting.
    #[serde(default)]
    pub dry_run: bool,
}

/// DELETE /v1/memories/outdated response.
#[derive(Debug, Serialize)]
pub struct DeleteOutdatedResponse {
    pub deleted: usize,
    pub dry_run: bool,
}

/// POST /v1/memories/merge request body.
#[derive(Debug, Deserialize)]
pub struct MergeMemoriesRequest {
    /// Memory IDs to merge.
    pub memory_ids: Vec<String>,
    /// Optional override content. If omitted, LLM summarizes.
    pub content: Option<String>,
    /// Tags for the merged memory (default: union of source tags).
    pub tags: Option<Vec<String>>,
    pub namespace: Option<String>,
    pub agent_id: Option<String>,
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
