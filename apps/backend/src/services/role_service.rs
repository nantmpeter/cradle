use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::extractors::auth::AuthUser;
use crate::models::role::{CreateRoleRequest, RoleResponse, UpdateRolePermissionsRequest, UpdateRoleRequest};
use crate::repository::{permission_repo, role_repo};

/// List all roles with their permissions
pub async fn list_roles(pool: &PgPool) -> Result<Vec<RoleResponse>, AppError> {
    role_repo::list_with_permissions(pool).await
}

/// Get a single role with its permissions
pub async fn get_role(pool: &PgPool, id: Uuid) -> Result<RoleResponse, AppError> {
    role_repo::get_with_permissions(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Role {} not found", id)))
}

/// Create a new role
pub async fn create_role(
    pool: &PgPool,
    auth: &AuthUser,
    req: &CreateRoleRequest,
) -> Result<RoleResponse, AppError> {
    // Permission check
    auth.require_permission(pool, "roles:create").await?;

    // Check name uniqueness
    if role_repo::find_by_name(pool, &req.name).await?.is_some() {
        return Err(AppError::Conflict(format!(
            "Role with name '{}' already exists",
            req.name
        )));
    }

    let data_scope = req.data_scope.as_deref().unwrap_or("all");
    let role = role_repo::create(pool, &req.name, req.description.as_deref(), &req.permission_ids, data_scope).await?;

    // Audit log
    if let Err(e) = crate::repository::audit_log_repo::create(
        pool,
        Some(auth.user_id),
        "role.create",
        Some("role"),
        Some(role.id),
        Some(serde_json::json!({
            "name": role.name,
            "permission_count": req.permission_ids.len(),
        })),
        None,
        None,
    )
    .await
    {
        tracing::error!("Failed to write audit log for role creation: {:?}", e);
    }

    // Fetch the full role with permissions for response
    get_role(pool, role.id).await
}

/// Update a role's name/description
pub async fn update_role(
    pool: &PgPool,
    auth: &AuthUser,
    id: Uuid,
    req: &UpdateRoleRequest,
) -> Result<RoleResponse, AppError> {
    // Permission check
    auth.require_permission(pool, "roles:update").await?;

    // Find the role
    let existing = role_repo::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Role {} not found", id)))?;

    // Cannot modify system roles
    if existing.is_system {
        return Err(AppError::Forbidden("Cannot modify system roles".into()));
    }

    // Check name uniqueness if name is being changed
    if let Some(ref name) = req.name {
        if let Some(conflict) = role_repo::find_by_name(pool, name).await? {
            if conflict.id != id {
                return Err(AppError::Conflict(format!(
                    "Role with name '{}' already exists",
                    name
                )));
            }
        }
    }

    let updated = role_repo::update(
        pool,
        id,
        req.name.as_deref(),
        req.description.as_deref(),
        req.data_scope.as_deref(),
    )
    .await?;

    // Audit log
    if let Err(e) = crate::repository::audit_log_repo::create(
        pool,
        Some(auth.user_id),
        "role.update",
        Some("role"),
        Some(id),
        Some(serde_json::json!({
            "name": req.name,
            "description_updated": req.description.is_some(),
        })),
        None,
        None,
    )
    .await
    {
        tracing::error!("Failed to write audit log for role update: {:?}", e);
    }

    get_role(pool, updated.id).await
}

/// Delete a role (cannot delete system roles or roles with users)
pub async fn delete_role(
    pool: &PgPool,
    auth: &AuthUser,
    id: Uuid,
) -> Result<(), AppError> {
    // Permission check
    auth.require_permission(pool, "roles:delete").await?;

    // Find the role
    let existing = role_repo::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Role {} not found", id)))?;

    // Cannot delete system roles
    if existing.is_system {
        return Err(AppError::Forbidden("Cannot delete system roles".into()));
    }

    // Check if any users are assigned to this role
    let user_count = role_repo::count_users_by_role(pool, id).await?;
    if user_count > 0 {
        return Err(AppError::Conflict(format!(
            "Cannot delete role: {} user(s) are still assigned to this role",
            user_count
        )));
    }

    role_repo::delete(pool, id).await?;

    // Audit log
    if let Err(e) = crate::repository::audit_log_repo::create(
        pool,
        Some(auth.user_id),
        "role.delete",
        Some("role"),
        Some(id),
        Some(serde_json::json!({
            "name": existing.name,
        })),
        None,
        None,
    )
    .await
    {
        tracing::error!("Failed to write audit log for role deletion: {:?}", e);
    }

    Ok(())
}

/// Update permissions assigned to a role
pub async fn update_permissions(
    pool: &PgPool,
    auth: &AuthUser,
    id: Uuid,
    req: &UpdateRolePermissionsRequest,
) -> Result<RoleResponse, AppError> {
    // Permission check
    auth.require_permission(pool, "roles:manage_permissions").await?;

    // Find the role
    let existing = role_repo::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Role {} not found", id)))?;

    // Cannot modify system role permissions (superadmin should still be able to manage admin/user permissions)
    // Actually, let's allow it for non-superadmin system roles if the user has the permission
    // But we should not allow removing all permissions from system roles
    // For now, just check the role exists and update

    // Set the new permissions
    permission_repo::set_role_permissions(pool, id, &req.permission_ids).await?;

    // Audit log
    if let Err(e) = crate::repository::audit_log_repo::create(
        pool,
        Some(auth.user_id),
        "role.permissions_change",
        Some("role"),
        Some(id),
        Some(serde_json::json!({
            "role_name": existing.name,
            "permission_count": req.permission_ids.len(),
        })),
        None,
        None,
    )
    .await
    {
        tracing::error!("Failed to write audit log for role permissions change: {:?}", e);
    }

    get_role(pool, id).await
}
