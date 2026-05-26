use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::error::AppError;
use crate::extractors::auth::AuthUser;
use crate::repository::{dashboard_repo, permission_repo, user_repo};
use crate::AppState;

/// GET /api/dashboard/stats
///
/// Returns dashboard statistics with field-level permission filtering.
/// - `users:read` → users stats + today_logins
/// - `roles:read` → roles.distribution
/// - `audit:read` → recent_audit_logs
/// - roles.total is visible to all authenticated users
/// - superadmin has all permissions
pub async fn stats(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<Value>, AppError> {
    let pool = &state.db;

    // Fetch user permissions
    let user_permissions = if auth.is_superadmin() {
        // superadmin gets all fields — use a sentinel to skip filtering
        vec!["*".to_string()]
    } else {
        permission_repo::find_by_user_id(pool, auth.user_id).await?
    };

    let has_perm = |perm: &str| -> bool {
        user_permissions.iter().any(|p| p == "*" || p == perm)
    };

    let has_users_read = has_perm("users:read");
    let has_roles_read = has_perm("roles:read");
    let has_audit_read = has_perm("audit:read");

    // Run all aggregation queries in parallel
    let (status_counts, role_dist, total_roles, today_logins, recent_logs) = tokio::join!(
        dashboard_repo::count_users_by_status(pool),
        dashboard_repo::count_users_by_role(pool),
        dashboard_repo::count_total_roles(pool),
        dashboard_repo::count_today_logins(pool),
        dashboard_repo::recent_audit_logs(pool, 5),
    );

    let status_counts = status_counts?;
    let role_dist = role_dist?;
    let total_roles = total_roles?;
    let today_logins = today_logins?;
    let recent_logs = recent_logs?;

    // Build users block (requires users:read)
    let users_value = if has_users_read {
        let mut total: i64 = 0;
        let mut active: i64 = 0;
        let mut disabled: i64 = 0;
        for row in &status_counts {
            total += row.count;
            match row.status.as_str() {
                "active" => active += row.count,
                "disabled" => disabled += row.count,
                _ => {}
            }
        }
        Some(json!({
            "total": total,
            "active": active,
            "disabled": disabled,
        }))
    } else {
        None
    };

    // Build roles.distribution (requires roles:read)
    let distribution_value = if has_roles_read {
        Some(
            role_dist
                .into_iter()
                .map(|r| json!({ "role_name": r.role_name, "count": r.count }))
                .collect::<Vec<_>>(),
        )
    } else {
        None
    };

    // Build recent_audit_logs (requires audit:read)
    let recent_logs_value = if has_audit_read {
        recent_logs
            .into_iter()
            .map(|log| {
                json!({
                    "id": log.id,
                    "action": log.action,
                    "user_email": log.user_email,
                    "resource_type": log.resource_type,
                    "created_at": log.created_at,
                })
            })
            .collect::<Vec<_>>()
    } else {
        vec![]
    };

    // today_logins requires users:read
    let today_logins_value = if has_users_read {
        json!(today_logins)
    } else {
        Value::Null
    };

    let response = json!({
        "users": users_value,
        "roles": {
            "total": total_roles,
            "distribution": distribution_value,
        },
        "today_logins": today_logins_value,
        "recent_audit_logs": recent_logs_value,
    });

    Ok(Json(response))
}

/// GET /api/dashboard/settings
///
/// Returns system settings information.
/// Requires `dashboard:read` permission.
/// - System info (name, version, uptime) is always included.
/// - Database info (status, max_connections) requires `system:manage`.
/// - Current user info is always included.
pub async fn settings(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<Value>, AppError> {
    let pool = &state.db;

    // Require dashboard:read permission
    auth.require_permission(pool, "dashboard:read").await?;

    // System info
    let uptime_secs = state.started_at.elapsed().as_secs();
    let system = json!({
        "name": "Cradle",
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_seconds": uptime_secs,
    });

    // Database info (requires system:manage)
    let database = if auth.is_superadmin() {
        // Check DB health with a simple query
        let db_healthy = sqlx::query("SELECT 1")
            .execute(pool)
            .await
            .is_ok();

        let status = if db_healthy {
            "connected"
        } else {
            "disconnected"
        };

        Some(json!({
            "status": status,
            "max_connections": state.max_connections,
        }))
    } else {
        // Check system:manage permission for non-superadmin
        let user_permissions = permission_repo::find_by_user_id(pool, auth.user_id).await?;
        if user_permissions.iter().any(|p| p == "system:manage") {
            let db_healthy = sqlx::query("SELECT 1")
                .execute(pool)
                .await
                .is_ok();

            let status = if db_healthy {
                "connected"
            } else {
                "disconnected"
            };

            Some(json!({
                "status": status,
                "max_connections": state.max_connections,
            }))
        } else {
            None
        }
    };

    // Current user info
    let user = user_repo::find_by_id(pool, auth.user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    let permissions = permission_repo::find_by_user_id(pool, auth.user_id).await?;

    let current_user = json!({
        "id": user.id,
        "email": user.email,
        "name": user.name,
        "role": user.role,
        "two_factor_enabled": user.two_factor_enabled,
        "permissions": permissions,
        "created_at": user.created_at,
    });

    let response = json!({
        "system": system,
        "database": database,
        "current_user": current_user,
    });

    Ok(Json(response))
}
