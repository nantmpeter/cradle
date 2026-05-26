use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::role::{RoleRecord, RoleResponse};

/// Find a role by its ID
pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<RoleRecord>, AppError> {
    let role = sqlx::query_as::<_, RoleRecord>("SELECT * FROM roles WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(role)
}

/// Find a role by its name
pub async fn find_by_name(pool: &PgPool, name: &str) -> Result<Option<RoleRecord>, AppError> {
    let role = sqlx::query_as::<_, RoleRecord>("SELECT * FROM roles WHERE name = $1")
        .bind(name)
        .fetch_optional(pool)
        .await?;
    Ok(role)
}

/// List all roles (without permissions)
pub async fn list_all(pool: &PgPool) -> Result<Vec<RoleRecord>, AppError> {
    let roles = sqlx::query_as::<_, RoleRecord>(
        "SELECT * FROM roles ORDER BY created_at ASC",
    )
    .fetch_all(pool)
    .await?;
    Ok(roles)
}

/// Helper struct for role+permission query result
#[derive(Debug, sqlx::FromRow)]
struct RolePermissionRow {
    id: Uuid,
    name: String,
    description: Option<String>,
    is_system: bool,
    data_scope: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    permission_name: Option<String>,
}

/// List all roles with their permissions
pub async fn list_with_permissions(pool: &PgPool) -> Result<Vec<RoleResponse>, AppError> {
    let rows = sqlx::query_as::<_, RolePermissionRow>(
        r#"
        SELECT r.id, r.name, r.description, r.is_system, r.data_scope, r.created_at, r.updated_at,
               p.name AS permission_name
        FROM roles r
        LEFT JOIN role_permissions rp ON r.id = rp.role_id
        LEFT JOIN permissions p ON rp.permission_id = p.id
        ORDER BY r.created_at ASC, p.name ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    // Group by role
    let mut role_map: std::collections::BTreeMap<Uuid, RoleResponse> =
        std::collections::BTreeMap::new();

    for row in rows {
        role_map
            .entry(row.id)
            .or_insert_with(|| RoleResponse {
                id: row.id,
                name: row.name.clone(),
                description: row.description.clone(),
                is_system: row.is_system,
                data_scope: row.data_scope.clone(),
                permissions: Vec::new(),
                created_at: row.created_at,
                updated_at: row.updated_at,
            });

        if let Some(perm_name) = row.permission_name {
            if let Some(role_resp) = role_map.get_mut(&row.id) {
                role_resp.permissions.push(perm_name);
            }
        }
    }

    Ok(role_map.into_values().collect())
}

/// Create a new role with the given permissions
pub async fn create(
    pool: &PgPool,
    name: &str,
    description: Option<&str>,
    permission_ids: &[Uuid],
    data_scope: &str,
) -> Result<RoleRecord, AppError> {
    let role = sqlx::query_as::<_, RoleRecord>(
        "INSERT INTO roles (name, description, data_scope) VALUES ($1, $2, $3) RETURNING *",
    )
    .bind(name)
    .bind(description)
    .bind(data_scope)
    .fetch_one(pool)
    .await?;

    // Insert role-permission associations
    for perm_id in permission_ids {
        sqlx::query(
            "INSERT INTO role_permissions (role_id, permission_id) VALUES ($1, $2)",
        )
        .bind(role.id)
        .bind(perm_id)
        .execute(pool)
        .await?;
    }

    Ok(role)
}

/// Update a role's name, description, and/or data_scope
pub async fn update(
    pool: &PgPool,
    id: Uuid,
    name: Option<&str>,
    description: Option<&str>,
    data_scope: Option<&str>,
) -> Result<RoleRecord, AppError> {
    let role = sqlx::query_as::<_, RoleRecord>(
        "UPDATE roles SET name = COALESCE($1, name), description = COALESCE($2, description), data_scope = COALESCE($3, data_scope), updated_at = NOW() WHERE id = $4 RETURNING *",
    )
    .bind(name)
    .bind(description)
    .bind(data_scope)
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(role)
}

/// Delete a role by ID
pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM roles WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Role {} not found", id)));
    }

    Ok(())
}

/// Count users assigned to a specific role
pub async fn count_users_by_role(pool: &PgPool, role_id: Uuid) -> Result<i64, AppError> {
    #[derive(sqlx::FromRow)]
    struct CountResult {
        count: i64,
    }

    let result: CountResult =
        sqlx::query_as::<_, CountResult>("SELECT COUNT(*) as count FROM users WHERE role_id = $1")
            .bind(role_id)
            .fetch_one(pool)
            .await?;

    Ok(result.count)
}

/// Get a single role with its permissions by ID
pub async fn get_with_permissions(pool: &PgPool, id: Uuid) -> Result<Option<RoleResponse>, AppError> {
    let roles = list_with_permissions(pool).await?;
    Ok(roles.into_iter().find(|r| r.id == id))
}
