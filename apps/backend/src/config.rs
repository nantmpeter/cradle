use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub server: ServerSettings,
    pub database: DatabaseSettings,
    pub jwt: JwtSettings,
    pub rate_limit: RateLimitSettings,
    #[serde(default)]
    pub storage: StorageSettings,
    #[serde(default)]
    pub totp: TotpSettings,
    #[serde(default)]
    pub api: ApiSettings,
    #[serde(default)]
    pub webhook: WebhookSettings,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseSettings {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JwtSettings {
    pub secret: String,
    pub access_exp_secs: i64,
    pub refresh_exp_secs: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RateLimitSettings {
    /// Global rate limit: max requests per window per IP
    #[serde(default = "default_global_rpm")]
    pub global_rpm: u64,
    /// Login rate limit: max login attempts per window per IP
    #[serde(default = "default_login_rpm")]
    pub login_rpm: u64,
}

fn default_global_rpm() -> u64 {
    100
}

fn default_login_rpm() -> u64 {
    5
}

impl Default for RateLimitSettings {
    fn default() -> Self {
        Self {
            global_rpm: default_global_rpm(),
            login_rpm: default_login_rpm(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct StorageSettings {
    #[serde(default = "default_upload_dir")]
    pub upload_dir: String,
    #[serde(default = "default_max_upload_size")]
    pub max_upload_size: u64,
    #[serde(default = "default_max_avatar_size")]
    pub max_avatar_size: u64,
}

fn default_upload_dir() -> String {
    "./uploads".to_string()
}

fn default_max_upload_size() -> u64 {
    10_485_760 // 10 MB
}

fn default_max_avatar_size() -> u64 {
    2_097_152 // 2 MB
}

impl Default for StorageSettings {
    fn default() -> Self {
        Self {
            upload_dir: default_upload_dir(),
            max_upload_size: default_max_upload_size(),
            max_avatar_size: default_max_avatar_size(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct TotpSettings {
    #[serde(default = "default_totp_encryption_key")]
    pub encryption_key: String,
}

fn default_totp_encryption_key() -> String {
    // Default key for development only; must be overridden in production
    "dGVzdC1rZXktZm9yLWRldmVsb3BtZW50LW9ubHk=".to_string()
}

impl Default for TotpSettings {
    fn default() -> Self {
        Self {
            encryption_key: default_totp_encryption_key(),
        }
    }
}

// ─── Phase 8: API & Webhook Settings ───

fn default_version() -> String {
    "v1".to_string()
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApiSettings {
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default = "default_true")]
    pub legacy_routes_enabled: bool,
    #[serde(default = "default_true")]
    pub docs_enabled: bool,
    #[serde(default)]
    pub wechat: WechatSettings,
    #[serde(default)]
    pub sms: SmsSettings,
    #[serde(default)]
    pub github: GithubSettings,
    #[serde(default)]
    pub google: GoogleSettings,
}

impl Default for ApiSettings {
    fn default() -> Self {
        Self {
            version: default_version(),
            legacy_routes_enabled: true,
            docs_enabled: true,
            wechat: WechatSettings::default(),
            sms: SmsSettings::default(),
            github: GithubSettings::default(),
            google: GoogleSettings::default(),
        }
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct WechatSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub app_id: String,
    #[serde(default)]
    pub app_secret: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct SmsSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub provider: String,
    #[serde(default)]
    pub template_id: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct GithubSettings {
    #[serde(default)]
    pub client_id: String,
    #[serde(default)]
    pub client_secret: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct GoogleSettings {
    #[serde(default)]
    pub client_id: String,
    #[serde(default)]
    pub client_secret: String,
}

fn default_max_retries() -> u32 {
    3
}

fn default_retry_intervals() -> Vec<u64> {
    vec![60, 300, 1800]
}

fn default_timeout() -> u64 {
    10
}

#[derive(Debug, Deserialize, Clone)]
pub struct WebhookSettings {
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    #[serde(default = "default_retry_intervals")]
    pub retry_intervals_secs: Vec<u64>,
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

impl Default for WebhookSettings {
    fn default() -> Self {
        Self {
            max_retries: default_max_retries(),
            retry_intervals_secs: default_retry_intervals(),
            timeout_secs: default_timeout(),
        }
    }
}

impl Settings {
    pub fn new() -> anyhow::Result<Self> {
        let config = config::Config::builder()
            .add_source(config::File::with_name("config/default").required(false))
            .add_source(config::File::with_name("config/local").required(false))
            .add_source(
                config::Environment::with_prefix("APP")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?;
        Ok(config.try_deserialize()?)
    }
}
