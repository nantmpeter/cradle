use sqlx::PgPool;

use crate::error::AppError;
use crate::models::system_config::SystemConfig;

/// Find all system configs
pub async fn find_all(pool: &PgPool) -> Result<Vec<SystemConfig>, AppError> {
    let configs = sqlx::query_as::<_, SystemConfig>(
        "SELECT * FROM system_configs ORDER BY group_key, config_key",
    )
    .fetch_all(pool)
    .await?;

    Ok(configs)
}

/// Find configs by group key
pub async fn find_by_group(pool: &PgPool, group_key: &str) -> Result<Vec<SystemConfig>, AppError> {
    let configs = sqlx::query_as::<_, SystemConfig>(
        "SELECT * FROM system_configs WHERE group_key = $1 ORDER BY config_key",
    )
    .bind(group_key)
    .fetch_all(pool)
    .await?;

    Ok(configs)
}

/// Find a specific config by group + key
pub async fn find_by_key(
    pool: &PgPool,
    group_key: &str,
    config_key: &str,
) -> Result<Option<SystemConfig>, AppError> {
    let config = sqlx::query_as::<_, SystemConfig>(
        "SELECT * FROM system_configs WHERE group_key = $1 AND config_key = $2",
    )
    .bind(group_key)
    .bind(config_key)
    .fetch_optional(pool)
    .await?;

    Ok(config)
}

/// Upsert a config: insert or update on conflict
pub async fn upsert(
    pool: &PgPool,
    group_key: &str,
    config_key: &str,
    value: &str,
    value_type: &str,
    description: Option<&str>,
) -> Result<SystemConfig, AppError> {
    let config = sqlx::query_as::<_, SystemConfig>(
        r#"
        INSERT INTO system_configs (group_key, config_key, value, value_type, description)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (group_key, config_key)
        DO UPDATE SET value = $3, value_type = $4, description = COALESCE($5, system_configs.description), updated_at = NOW()
        RETURNING *
        "#,
    )
    .bind(group_key)
    .bind(config_key)
    .bind(value)
    .bind(value_type)
    .bind(description)
    .fetch_one(pool)
    .await?;

    Ok(config)
}

/// Delete a config by group + key
pub async fn delete(pool: &PgPool, group_key: &str, config_key: &str) -> Result<(), AppError> {
    let result = sqlx::query(
        "DELETE FROM system_configs WHERE group_key = $1 AND config_key = $2",
    )
    .bind(group_key)
    .bind(config_key)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!(
            "Config {}/{} not found",
            group_key, config_key
        )));
    }

    Ok(())
}

/// Find public configs (general group only)
pub async fn find_public(pool: &PgPool) -> Result<Vec<SystemConfig>, AppError> {
    let configs = sqlx::query_as::<_, SystemConfig>(
        "SELECT * FROM system_configs WHERE group_key = 'general' ORDER BY config_key",
    )
    .fetch_all(pool)
    .await?;

    Ok(configs)
}
