use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;

// ─── FromRow structs for aggregation queries ────────────────────────────────

/// Result row for user count grouped by status
#[derive(sqlx::FromRow)]
pub struct UserStatusCount {
    pub status: String,
    pub count: i64,
}

/// Result row for role distribution (role name + user count)
#[derive(sqlx::FromRow)]
pub struct RoleDistributionRow {
    pub role_name: String,
    pub count: i64,
}

/// Summary of a recent audit log entry for the dashboard
#[derive(sqlx::FromRow)]
pub struct RecentAuditLogSummary {
    pub id: Uuid,
    pub action: String,
    pub user_email: Option<String>,
    pub resource_type: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Helper struct for scalar count queries
#[derive(sqlx::FromRow)]
struct CountResult {
    count: i64,
}

/// Count users grouped by status
pub async fn count_users_by_status(pool: &PgPool) -> Result<Vec<UserStatusCount>, AppError> {
    let rows = sqlx::query_as::<_, UserStatusCount>(
        "SELECT status, COUNT(*) as count FROM users GROUP BY status",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Count users grouped by role name (includes roles with zero users)
pub async fn count_users_by_role(pool: &PgPool) -> Result<Vec<RoleDistributionRow>, AppError> {
    let rows = sqlx::query_as::<_, RoleDistributionRow>(
        r#"
        SELECT r.name as role_name, COUNT(u.id) as count
        FROM roles r
        LEFT JOIN users u ON u.role_id = r.id
        GROUP BY r.name
        ORDER BY count DESC
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Count total number of roles
pub async fn count_total_roles(pool: &PgPool) -> Result<i64, AppError> {
    let row: CountResult =
        sqlx::query_as::<_, CountResult>("SELECT COUNT(*) as count FROM roles")
            .fetch_one(pool)
            .await?;
    Ok(row.count)
}

/// Count today's login events (action = 'auth.login')
pub async fn count_today_logins(pool: &PgPool) -> Result<i64, AppError> {
    let row: CountResult = sqlx::query_as::<_, CountResult>(
        "SELECT COUNT(*) as count FROM audit_logs WHERE action = 'auth.login' AND created_at >= CURRENT_DATE",
    )
    .fetch_one(pool)
    .await?;
    Ok(row.count)
}

/// Fetch the most recent N audit log entries with user email
pub async fn recent_audit_logs(
    pool: &PgPool,
    limit: i64,
) -> Result<Vec<RecentAuditLogSummary>, AppError> {
    let rows = sqlx::query_as::<_, RecentAuditLogSummary>(
        r#"
        SELECT al.id, al.action, u.email AS user_email, al.resource_type, al.created_at
        FROM audit_logs al
        LEFT JOIN users u ON al.user_id = u.id
        ORDER BY al.created_at DESC
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
