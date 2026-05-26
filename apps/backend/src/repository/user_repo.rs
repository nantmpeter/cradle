use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::user::User;

/// Parameters for paginated, filtered user listing
pub struct ListParams {
    pub search: Option<String>,
    pub role: Option<String>,
    pub role_id: Option<Uuid>,
    pub status: Option<String>,
    pub sort_by: String,
    pub sort_order: String,
    pub page: i64,
    pub per_page: i64,
    /// If set, filter users to only those in these department IDs
    pub department_ids: Option<Vec<Uuid>>,
}

/// Helper struct for count query result
#[derive(sqlx::FromRow)]
struct CountResult {
    count: i64,
}

/// Apply filter conditions to a QueryBuilder
fn apply_filters(qb: &mut QueryBuilder<Postgres>, params: &ListParams) {
    if let Some(ref search) = params.search {
        let pattern = format!("%{}%", search);
        qb.push(" AND (email ILIKE ");
        qb.push_bind(pattern.clone());
        qb.push(" OR COALESCE(name, '') ILIKE ");
        qb.push_bind(pattern);
        qb.push(")");
    }
    if let Some(ref role) = params.role {
        qb.push(" AND role = ");
        qb.push_bind(role.clone());
    }
    if let Some(role_id) = params.role_id {
        qb.push(" AND role_id = ");
        qb.push_bind(role_id);
    }
    if let Some(ref status) = params.status {
        qb.push(" AND status = ");
        qb.push_bind(status.clone());
    }
    if let Some(ref dept_ids) = params.department_ids {
        if dept_ids.is_empty() {
            // No departments visible = only self (handled in handler layer)
            qb.push(" AND 1=0");
        } else {
            qb.push(" AND department_id = ANY(");
            qb.push_bind(dept_ids.clone());
            qb.push(")");
        }
    }
}

/// List users with dynamic filtering, sorting, and pagination
pub async fn list_users_paginated(
    pool: &PgPool,
    params: &ListParams,
) -> Result<(Vec<User>, i64), AppError> {
    // Data query
    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new("SELECT * FROM users WHERE 1=1");
    apply_filters(&mut qb, params);

    let sort_col = match params.sort_by.as_str() {
        s @ ("email" | "name" | "role" | "status" | "created_at") => s,
        _ => "created_at",
    };
    let sort_dir = match params.sort_order.as_str() {
        "asc" => "ASC",
        _ => "DESC",
    };
    // sort_col is validated against a whitelist; safe to interpolate
    qb.push(format!(" ORDER BY {} {}", sort_col, sort_dir));
    qb.push(" LIMIT ");
    qb.push_bind(params.per_page);
    qb.push(" OFFSET ");
    qb.push_bind((params.page - 1) * params.per_page);

    let users = qb.build_query_as::<User>().fetch_all(pool).await?;

    // Count query
    let mut cq: QueryBuilder<Postgres> =
        QueryBuilder::new("SELECT COUNT(*) as count FROM users WHERE 1=1");
    apply_filters(&mut cq, params);

    let count_result: CountResult = cq.build_query_as::<CountResult>().fetch_one(pool).await?;

    Ok((users, count_result.count))
}

pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, AppError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(pool)
        .await?;
    Ok(user)
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>, AppError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(user)
}

/// Count superadmin users
pub async fn count_superadmins(pool: &PgPool) -> Result<i64, AppError> {
    let row: CountResult =
        sqlx::query_as::<_, CountResult>("SELECT COUNT(*) as count FROM users WHERE role = 'superadmin'")
            .fetch_one(pool)
            .await?;
    Ok(row.count)
}

/// Create a new user with explicit role, status, and must_change_password
pub async fn create_user(
    pool: &PgPool,
    email: &str,
    password_hash: &str,
    name: Option<&str>,
    role: &str,
    status: &str,
    must_change_password: bool,
    role_id: Option<Uuid>,
    department_id: Option<Uuid>,
) -> Result<User, AppError> {
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (email, password_hash, name, role, status, must_change_password, role_id, department_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *",
    )
    .bind(email)
    .bind(password_hash)
    .bind(name)
    .bind(role)
    .bind(status)
    .bind(must_change_password)
    .bind(role_id)
    .bind(department_id)
    .fetch_one(pool)
    .await?;

    Ok(user)
}

/// Update a user's name and/or role
pub async fn update_user(
    pool: &PgPool,
    id: Uuid,
    name: Option<&str>,
    role: Option<&str>,
    role_id: Option<Uuid>,
    department_id: Option<Option<Uuid>>,
) -> Result<User, AppError> {
    // If department_id is Some(Some(id)) or Some(None), update it
    // If department_id is None, don't change it
    let user = if let Some(dept_id) = department_id {
        sqlx::query_as::<_, User>(
            "UPDATE users SET name = COALESCE($1, name), role = COALESCE($2, role), role_id = COALESCE($3, role_id), department_id = $4, updated_at = NOW() WHERE id = $5 RETURNING *",
        )
        .bind(name)
        .bind(role)
        .bind(role_id)
        .bind(dept_id)
        .bind(id)
        .fetch_one(pool)
        .await?
    } else {
        sqlx::query_as::<_, User>(
            "UPDATE users SET name = COALESCE($1, name), role = COALESCE($2, role), role_id = COALESCE($3, role_id), updated_at = NOW() WHERE id = $4 RETURNING *",
        )
        .bind(name)
        .bind(role)
        .bind(role_id)
        .bind(id)
        .fetch_one(pool)
        .await?
    };

    Ok(user)
}

/// Update a user's status (active/disabled)
pub async fn update_status(pool: &PgPool, id: Uuid, status: &str) -> Result<User, AppError> {
    let user = sqlx::query_as::<_, User>(
        "UPDATE users SET status = $1, updated_at = NOW() WHERE id = $2 RETURNING *",
    )
    .bind(status)
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(user)
}

/// Update a user's password and optionally set must_change_password
pub async fn update_password(
    pool: &PgPool,
    id: Uuid,
    password_hash: &str,
    must_change_password: bool,
) -> Result<User, AppError> {
    let user = sqlx::query_as::<_, User>(
        "UPDATE users SET password_hash = $1, must_change_password = $2, updated_at = NOW() WHERE id = $3 RETURNING *",
    )
    .bind(password_hash)
    .bind(must_change_password)
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(user)
}

pub async fn delete_user(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("User {} not found", id)));
    }
    Ok(())
}

/// Increment login failure count and return the new count
pub async fn increment_login_failures(pool: &PgPool, id: Uuid) -> Result<i32, AppError> {
    #[derive(sqlx::FromRow)]
    struct FailResult {
        login_failures: Option<i32>,
    }

    let result: FailResult = sqlx::query_as::<_, FailResult>(
        "UPDATE users SET login_failures = COALESCE(login_failures, 0) + 1 WHERE id = $1 RETURNING login_failures",
    )
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(result.login_failures.unwrap_or(0))
}

/// Reset login failure count and clear locked_until
pub async fn reset_login_failures(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE users SET login_failures = 0, locked_until = NULL WHERE id = $1",
    )
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Set the locked_until timestamp for a user
pub async fn set_locked_until(
    pool: &PgPool,
    id: Uuid,
    locked_until: chrono::DateTime<chrono::Utc>,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE users SET locked_until = $1 WHERE id = $2",
    )
    .bind(locked_until)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Update user avatar URL
pub async fn update_avatar_url(
    pool: &PgPool,
    id: Uuid,
    avatar_url: Option<&str>,
) -> Result<(), AppError> {
    sqlx::query("UPDATE users SET avatar_url = $1, updated_at = NOW() WHERE id = $2")
        .bind(avatar_url)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
