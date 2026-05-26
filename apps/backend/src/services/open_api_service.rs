use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::repository::{department_repo, user_repo};

// ─── Open API Response Types ────────────────────────────────────────────────

/// Public user info for the Open API (excludes sensitive fields)
#[derive(Debug, Serialize)]
pub struct OpenApiUser {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub role: String,
    pub status: String,
    pub department_id: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Public department info for the Open API
#[derive(Debug, Serialize)]
pub struct OpenApiDepartment {
    pub id: Uuid,
    pub name: String,
    pub code: String,
    pub parent_id: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

// ─── Service Functions ──────────────────────────────────────────────────────

/// List users for the Open API.
/// Returns a simplified user view without sensitive fields.
pub async fn list_users(
    pool: &PgPool,
    page: i64,
    per_page: i64,
) -> Result<(Vec<OpenApiUser>, i64), AppError> {
    let page = page.max(1);
    let per_page = per_page.clamp(1, 100);

    let params = crate::repository::user_repo::ListParams {
        search: None,
        role: None,
        role_id: None,
        status: None,
        sort_by: "created_at".to_string(),
        sort_order: "desc".to_string(),
        page,
        per_page,
        department_ids: None,
    };

    let (users, total) = user_repo::list_users_paginated(pool, &params).await?;

    let open_users: Vec<OpenApiUser> = users
        .into_iter()
        .map(|u| OpenApiUser {
            id: u.id,
            email: u.email,
            name: u.name,
            role: u.role,
            status: u.status,
            department_id: u.department_id,
            created_at: u.created_at,
        })
        .collect();

    Ok((open_users, total))
}

/// Get a single user for the Open API.
pub async fn get_user(pool: &PgPool, user_id: Uuid) -> Result<Option<OpenApiUser>, AppError> {
    let user = user_repo::find_by_id(pool, user_id).await?;

    Ok(user.map(|u| OpenApiUser {
        id: u.id,
        email: u.email,
        name: u.name,
        role: u.role,
        status: u.status,
        department_id: u.department_id,
        created_at: u.created_at,
    }))
}

/// List departments for the Open API.
pub async fn list_departments(pool: &PgPool) -> Result<Vec<OpenApiDepartment>, AppError> {
    let departments = department_repo::list_all(pool).await?;

    let open_depts: Vec<OpenApiDepartment> = departments
        .into_iter()
        .map(|d| OpenApiDepartment {
            id: d.id,
            name: d.name,
            code: d.code,
            parent_id: d.parent_id,
            created_at: d.created_at,
        })
        .collect();

    Ok(open_depts)
}
