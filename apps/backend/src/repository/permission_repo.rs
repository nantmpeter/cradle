use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::permission::{PermissionRecord, PermissionResponse};

/// List all permissions
pub async fn list_all(pool: &PgPool) -> Result<Vec<PermissionRecord>, AppError> {
    let permissions = sqlx::query_as::<_, PermissionRecord>(
        "SELECT * FROM permissions ORDER BY module, name",
    )
    .fetch_all(pool)
    .await?;
    Ok(permissions)
}

/// Find permissions assigned to a specific role
pub async fn find_by_role_id(pool: &PgPool, role_id: Uuid) -> Result<Vec<PermissionResponse>, AppError> {
    let permissions = sqlx::query_as::<_, PermissionResponse>(
        r#"
        SELECT p.id, p.name, p.description, p.module, p.created_at
        FROM permissions p
        INNER JOIN role_permissions rp ON p.id = rp.permission_id
        WHERE rp.role_id = $1
        ORDER BY p.module, p.name
        "#,
    )
    .bind(role_id)
    .fetch_all(pool)
    .await?;
    Ok(permissions)
}

/// Find permission names assigned to a user (via their role_id)
pub async fn find_by_user_id(pool: &PgPool, user_id: Uuid) -> Result<Vec<String>, AppError> {
    let rows: Vec<(String,)> = sqlx::query_as(
        r#"
        SELECT p.name
        FROM permissions p
        INNER JOIN role_permissions rp ON p.id = rp.permission_id
        INNER JOIN users u ON u.role_id = rp.role_id
        WHERE u.id = $1
        ORDER BY p.name
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|(name,)| name).collect())
}

/// Set (replace) all permissions for a role
pub async fn set_role_permissions(
    pool: &PgPool,
    role_id: Uuid,
    permission_ids: &[Uuid],
) -> Result<(), AppError> {
    // Delete existing associations
    sqlx::query("DELETE FROM role_permissions WHERE role_id = $1")
        .bind(role_id)
        .execute(pool)
        .await?;

    // Insert new associations
    for perm_id in permission_ids {
        sqlx::query(
            "INSERT INTO role_permissions (role_id, permission_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(role_id)
        .bind(perm_id)
        .execute(pool)
        .await?;
    }

    Ok(())
}
