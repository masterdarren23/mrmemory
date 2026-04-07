use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;

/// Get the policy for a namespace. Returns None if no policy is set (defaults to "open").
pub async fn get_namespace_policy(
    db: &PgPool,
    tenant_id: Uuid,
    namespace: &str,
) -> Result<Option<String>, AppError> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT policy FROM namespace_policies WHERE tenant_id = $1 AND namespace = $2",
    )
    .bind(tenant_id)
    .bind(namespace)
    .fetch_optional(db)
    .await?;

    Ok(row.map(|(p,)| p))
}

/// Set (upsert) the policy for a namespace.
pub async fn set_namespace_policy(
    db: &PgPool,
    tenant_id: Uuid,
    namespace: &str,
    policy: &str,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO namespace_policies (tenant_id, namespace, policy) \
         VALUES ($1, $2, $3) \
         ON CONFLICT (tenant_id, namespace) DO UPDATE SET policy = $3",
    )
    .bind(tenant_id)
    .bind(namespace)
    .bind(policy)
    .execute(db)
    .await?;

    Ok(())
}

/// Check write permission based on namespace policy.
/// Returns Ok(()) if allowed, or an AppError::Forbidden if not.
pub async fn check_write_permission(
    db: &PgPool,
    tenant_id: Uuid,
    namespace: &str,
    agent_id: Option<&str>,
    memory_created_by: Option<&str>,
    is_create: bool,
) -> Result<(), AppError> {
    let policy = get_namespace_policy(db, tenant_id, namespace)
        .await?
        .unwrap_or_else(|| "open".into());

    match policy.as_str() {
        "open" => Ok(()),
        "append_only" => {
            if is_create {
                // Creates are always allowed in append_only
                Ok(())
            } else {
                // Updates/deletes only if agent_id matches memory's created_by
                let aid = agent_id.unwrap_or("unknown");
                let mcb = memory_created_by.unwrap_or("unknown");
                if aid == mcb {
                    Ok(())
                } else {
                    Err(AppError::Forbidden(format!(
                        "namespace '{}' has append_only policy: only the creator can modify/delete their memories",
                        namespace
                    )))
                }
            }
        }
        "read_only" => Err(AppError::Forbidden(format!(
            "namespace '{}' has read_only policy: no writes allowed",
            namespace
        ))),
        _ => Ok(()),
    }
}
