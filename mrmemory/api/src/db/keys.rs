use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::keys::{generate_api_key, hash_api_key, key_prefix};
use crate::error::AppError;

/// Create a new API key in the database. Returns (external_id, raw_key).
pub async fn create_api_key(
    db: &PgPool,
    tenant_id: Uuid,
    name: &str,
    scopes: &[String],
) -> Result<(String, String), AppError> {
    let id = Uuid::new_v4();
    let external_id = format!(
        "key_{}",
        id.simple().to_string().get(..12).unwrap_or("000000000000")
    );
    let raw_key = generate_api_key();
    let hash = hash_api_key(&raw_key);
    let prefix = key_prefix(&raw_key);

    sqlx::query(
        "INSERT INTO api_keys (id, external_id, tenant_id, name, key_hash, key_prefix, scopes) VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
    .bind(id)
    .bind(&external_id)
    .bind(tenant_id)
    .bind(name)
    .bind(&hash)
    .bind(&prefix)
    .bind(scopes)
    .execute(db)
    .await?;

    Ok((external_id, raw_key))
}
