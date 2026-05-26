use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use sqlx::PgPool;

use crate::error::AppError;
use crate::models::system_config::SystemConfigResponse;
use crate::repository::system_config_repo;

/// List all configs, optionally filtered by group
pub async fn list_by_group(
    pool: &PgPool,
    group_key: Option<&str>,
) -> Result<Vec<SystemConfigResponse>, AppError> {
    let configs = match group_key {
        Some(group) => system_config_repo::find_by_group(pool, group).await?,
        None => system_config_repo::find_all(pool).await?,
    };

    Ok(configs.into_iter().map(SystemConfigResponse::from).collect())
}

/// Get a single config by group and key
pub async fn get(
    pool: &PgPool,
    group_key: &str,
    config_key: &str,
) -> Result<Option<SystemConfigResponse>, AppError> {
    let config = system_config_repo::find_by_key(pool, group_key, config_key).await?;
    Ok(config.map(SystemConfigResponse::from))
}

/// Upsert a config value with type validation
pub async fn upsert(
    pool: &PgPool,
    group_key: &str,
    config_key: &str,
    value: &str,
    value_type: &str,
    description: Option<&str>,
    user_id: uuid::Uuid,
    config_cache: &Arc<RwLock<HashMap<String, HashMap<String, String>>>>,
) -> Result<SystemConfigResponse, AppError> {
    // Validate value_type
    validate_value_type(value, value_type)?;

    // Get old value for audit
    let old_config = system_config_repo::find_by_key(pool, group_key, config_key).await?;

    let new_config = system_config_repo::upsert(
        pool,
        group_key,
        config_key,
        value,
        value_type,
        description,
    )
    .await?;

    // Audit log
    let audit_details = serde_json::json!({
        "group_key": group_key,
        "config_key": config_key,
        "old_value": old_config.as_ref().map(|c| &c.value),
        "new_value": value,
        "value_type": value_type,
    });

    if let Err(e) = crate::repository::audit_log_repo::create(
        pool,
        Some(user_id),
        "config.update",
        Some("system_config"),
        Some(new_config.id),
        Some(audit_details),
        None,
        None,
    )
    .await
    {
        tracing::error!("Failed to write audit log for config update: {:?}", e);
    }

    // Refresh cache
    refresh_cache(config_cache, pool).await?;

    Ok(SystemConfigResponse::from(new_config))
}

/// Delete a config
pub async fn delete_config(
    pool: &PgPool,
    group_key: &str,
    config_key: &str,
    user_id: uuid::Uuid,
    config_cache: &Arc<RwLock<HashMap<String, HashMap<String, String>>>>,
) -> Result<(), AppError> {
    system_config_repo::delete(pool, group_key, config_key).await?;

    // Audit log
    if let Err(e) = crate::repository::audit_log_repo::create(
        pool,
        Some(user_id),
        "config.delete",
        Some("system_config"),
        None,
        Some(serde_json::json!({
            "group_key": group_key,
            "config_key": config_key,
        })),
        None,
        None,
    )
    .await
    {
        tracing::error!("Failed to write audit log for config delete: {:?}", e);
    }

    // Refresh cache
    refresh_cache(config_cache, pool).await?;

    Ok(())
}

/// Get public configs (general group only)
pub async fn get_public_configs(pool: &PgPool) -> Result<HashMap<String, String>, AppError> {
    let configs = system_config_repo::find_public(pool).await?;
    let mut map = HashMap::new();
    for c in configs {
        map.insert(c.config_key, c.value);
    }
    Ok(map)
}

/// Refresh the config cache from database
pub async fn refresh_cache(
    cache: &Arc<RwLock<HashMap<String, HashMap<String, String>>>>,
    pool: &PgPool,
) -> Result<(), AppError> {
    let configs = system_config_repo::find_all(pool).await?;
    let mut map: HashMap<String, HashMap<String, String>> = HashMap::new();
    for c in &configs {
        map.entry(c.group_key.clone())
            .or_default()
            .insert(c.config_key.clone(), c.value.clone());
    }

    if let Ok(mut cache) = cache.write() {
        *cache = map;
    }

    Ok(())
}

/// Validate that the value matches the declared value_type
fn validate_value_type(value: &str, value_type: &str) -> Result<(), AppError> {
    match value_type {
        "string" => Ok(()),
        "number" => {
            if value.parse::<f64>().is_err() {
                return Err(AppError::BadRequest(format!(
                    "Value '{}' is not a valid number",
                    value
                )));
            }
            Ok(())
        }
        "boolean" => {
            if value != "true" && value != "false" {
                return Err(AppError::BadRequest(format!(
                    "Value '{}' is not a valid boolean (true/false)",
                    value
                )));
            }
            Ok(())
        }
        "json" => {
            if serde_json::from_str::<serde_json::Value>(value).is_err() {
                return Err(AppError::BadRequest(format!(
                    "Value '{}' is not valid JSON",
                    value
                )));
            }
            Ok(())
        }
        _ => Err(AppError::BadRequest(format!(
            "Unknown value_type: {}",
            value_type
        ))),
    }
}
