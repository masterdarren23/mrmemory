use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};

use crate::auth::TenantContext;
use crate::error::AppError;
use crate::state::AppState;

/// GET /v1/stats — usage and observability stats for a tenant.
async fn get_stats(
    tenant: TenantContext,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let tenant_id = tenant.tenant_id;

    // Total memories
    let (total_memories,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM memories WHERE tenant_id = $1")
            .bind(tenant_id)
            .fetch_one(&state.db)
            .await?;

    // Compressed memories
    let (compressed_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM memories WHERE tenant_id = $1 AND is_compressed = TRUE")
            .bind(tenant_id)
            .fetch_one(&state.db)
            .await?;

    // Memories by namespace
    let namespace_rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT namespace, COUNT(*) FROM memories WHERE tenant_id = $1 GROUP BY namespace ORDER BY count DESC LIMIT 20"
    )
    .bind(tenant_id)
    .fetch_all(&state.db)
    .await?;

    let namespaces: Vec<serde_json::Value> = namespace_rows
        .iter()
        .map(|(ns, count)| serde_json::json!({"namespace": ns, "count": count}))
        .collect();

    // Memories by agent
    let agent_rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT agent_id, COUNT(*) FROM memories WHERE tenant_id = $1 GROUP BY agent_id ORDER BY count DESC LIMIT 20"
    )
    .bind(tenant_id)
    .fetch_all(&state.db)
    .await?;

    let agents: Vec<serde_json::Value> = agent_rows
        .iter()
        .map(|(aid, count)| serde_json::json!({"agent_id": aid, "count": count}))
        .collect();

    // Memory growth (last 30 days)
    let growth_rows: Vec<(chrono::NaiveDate, i64)> = sqlx::query_as(
        "SELECT created_at::date as day, COUNT(*) FROM memories WHERE tenant_id = $1 AND created_at > NOW() - INTERVAL '30 days' GROUP BY day ORDER BY day"
    )
    .bind(tenant_id)
    .fetch_all(&state.db)
    .await?;

    let growth: Vec<serde_json::Value> = growth_rows
        .iter()
        .map(|(day, count)| serde_json::json!({"date": day.to_string(), "created": count}))
        .collect();

    // Oldest and newest memory
    let oldest: Option<(chrono::DateTime<chrono::Utc>,)> = sqlx::query_as(
        "SELECT MIN(created_at) FROM memories WHERE tenant_id = $1"
    )
    .bind(tenant_id)
    .fetch_optional(&state.db)
    .await?;

    let newest: Option<(chrono::DateTime<chrono::Utc>,)> = sqlx::query_as(
        "SELECT MAX(created_at) FROM memories WHERE tenant_id = $1"
    )
    .bind(tenant_id)
    .fetch_optional(&state.db)
    .await?;

    // Total merged (sum of merged_from arrays)
    let (total_merged_sources,): (i64,) = sqlx::query_as(
        "SELECT COALESCE(SUM(array_length(merged_from, 1)), 0) FROM memories WHERE tenant_id = $1 AND is_compressed = TRUE"
    )
    .bind(tenant_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(serde_json::json!({
        "total_memories": total_memories,
        "compressed_memories": compressed_count,
        "compression_ratio": if total_memories > 0 {
            format!("{:.1}%", (compressed_count as f64 / total_memories as f64) * 100.0)
        } else {
            "0%".to_string()
        },
        "total_merged_sources": total_merged_sources,
        "memories_saved_by_compression": total_merged_sources - compressed_count,
        "namespaces": namespaces,
        "agents": agents,
        "growth_30d": growth,
        "oldest_memory": oldest.map(|o| o.0),
        "newest_memory": newest.map(|n| n.0),
        "plan": tenant.plan,
    })))
}

pub fn stats_routes() -> Router<AppState> {
    Router::new().route("/v1/stats", get(get_stats))
}
