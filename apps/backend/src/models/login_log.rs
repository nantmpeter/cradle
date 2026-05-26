use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 登录日志数据库模型
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct LoginLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub email: Option<String>,
    pub event: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub os: Option<String>,
    pub browser: Option<String>,
    pub success: bool,
    pub fail_reason: Option<String>,
    pub login_at: Option<DateTime<Utc>>,
}

/// 登录日志响应 DTO
#[derive(Debug, Serialize)]
pub struct LoginLogResponse {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub email: Option<String>,
    pub event: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub os: Option<String>,
    pub browser: Option<String>,
    pub success: bool,
    pub fail_reason: Option<String>,
    pub login_at: Option<DateTime<Utc>>,
}

impl From<LoginLog> for LoginLogResponse {
    fn from(log: LoginLog) -> Self {
        Self {
            id: log.id,
            user_id: log.user_id,
            email: log.email,
            event: log.event,
            ip_address: log.ip_address,
            user_agent: log.user_agent,
            os: log.os,
            browser: log.browser,
            success: log.success,
            fail_reason: log.fail_reason,
            login_at: log.login_at,
        }
    }
}

/// 创建登录日志参数（用于插入）
#[derive(Debug)]
pub struct CreateLoginLogParams {
    pub user_id: Option<Uuid>,
    pub email: String,
    pub event: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub os: Option<String>,
    pub browser: Option<String>,
    pub success: bool,
    pub fail_reason: Option<String>,
}

/// 登录日志列表查询参数
#[derive(Debug, Deserialize)]
pub struct LoginLogListParams {
    pub email: Option<String>,
    pub event: Option<String>,
    pub ip_address: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub page: i64,
    pub per_page: i64,
}
