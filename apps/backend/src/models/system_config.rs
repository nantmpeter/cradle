use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Database model for system_configs table
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SystemConfig {
    pub id: Uuid,
    pub group_key: String,
    pub config_key: String,
    pub value: String,
    pub value_type: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Public response for system config
#[derive(Debug, Serialize)]
pub struct SystemConfigResponse {
    pub group_key: String,
    pub config_key: String,
    pub value: String,
    pub value_type: String,
    pub description: Option<String>,
    pub updated_at: DateTime<Utc>,
}

impl From<SystemConfig> for SystemConfigResponse {
    fn from(c: SystemConfig) -> Self {
        Self {
            group_key: c.group_key,
            config_key: c.config_key,
            value: c.value,
            value_type: c.value_type,
            description: c.description,
            updated_at: c.updated_at,
        }
    }
}

/// Request to upsert a config value
#[derive(Debug, Deserialize)]
pub struct UpsertConfigRequest {
    pub value: String,
    #[serde(default = "default_value_type")]
    pub value_type: String,
    pub description: Option<String>,
}

fn default_value_type() -> String {
    "string".to_string()
}

/// Query for listing configs by group
#[derive(Debug, Deserialize)]
pub struct ConfigGroupQuery {
    pub group_key: Option<String>,
}
