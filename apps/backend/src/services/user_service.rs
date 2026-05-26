use uuid::Uuid;

use crate::error::AppError;
use crate::extractors::auth::AuthUser;
use crate::models::user::{
    validate_password_strength, CreateUserRequest, Role, UpdateUserRequest, UpdateStatusRequest,
    ChangeMyPasswordRequest, ResetPasswordRequest, MeResponse, UserResponse, UserStatus,
};
use crate::repository::{permission_repo, refresh_token_repo, user_repo};
use crate::services::auth_service;
use sqlx::PgPool;

/// Get a single user by ID
pub async fn get_user(pool: &PgPool, id: Uuid) -> Result<UserResponse, AppError> {
    let user = user_repo::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("User {} not found", id)))?;
    Ok(UserResponse::from(user))
}

/// Get the current user with permissions
pub async fn get_me(pool: &PgPool, user_id: Uuid) -> Result<MeResponse, AppError> {
    let user = user_repo::find_by_id(pool, user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    let permissions = permission_repo::find_by_user_id(pool, user_id).await?;

    Ok(MeResponse {
        id: user.id,
        email: user.email,
        name: user.name,
        role: user.role.parse().unwrap_or(Role::User),
        role_id: user.role_id.unwrap_or(uuid::Uuid::nil()),
        avatar_url: user.avatar_url,
        status: user.status.parse().unwrap_or(UserStatus::Active),
        must_change_password: user.must_change_password,
        two_factor_enabled: user.two_factor_enabled,
        department_id: user.department_id,
        permissions,
        created_at: user.created_at,
        updated_at: user.updated_at,
    })
}

/// List users with filtering, sorting, and pagination
pub async fn list_users(
    pool: &PgPool,
    auth: &AuthUser,
    search: Option<String>,
    role: Option<String>,
    role_id: Option<Uuid>,
    status: Option<String>,
    sort_by: Option<String>,
    sort_order: Option<String>,
    page: i64,
    per_page: i64,
) -> Result<(Vec<UserResponse>, i64), AppError> {
    // Permission check
    auth.require_permission(pool, "users:read").await?;

    // Data scope filtering
    let department_ids = auth.get_visible_department_ids(pool).await;

    let params = user_repo::ListParams {
        search,
        role,
        role_id,
        status,
        sort_by: sort_by.unwrap_or_else(|| "created_at".into()),
        sort_order: sort_order.unwrap_or_else(|| "desc".into()),
        page: page.max(1),
        per_page: per_page.clamp(1, 100),
        department_ids,
    };

    let (users, total) = user_repo::list_users_paginated(pool, &params).await?;
    let user_responses: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();

    Ok((user_responses, total))
}

/// Create a new user (admin+)
pub async fn create_user(
    pool: &PgPool,
    auth: &AuthUser,
    req: &CreateUserRequest,
) -> Result<UserResponse, AppError> {
    // Permission check
    auth.require_permission(pool, "users:create").await?;

    // Only superadmin can create superadmin
    if req.role == Role::SuperAdmin && !auth.is_superadmin() {
        return Err(AppError::Forbidden(
            "Only SuperAdmin can create superadmin accounts".into(),
        ));
    }

    // Enforce superadmin limit of 1
    if req.role == Role::SuperAdmin {
        let superadmin_count = user_repo::count_superadmins(pool).await?;
        if superadmin_count >= 1 {
            return Err(AppError::BadRequest(
                "SuperAdmin limit reached (maximum 1)".into(),
            ));
        }
    }

    // Validate password strength
    validate_password_strength(&req.password)
        .map_err(|e| AppError::BadRequest(e))?;

    // Check email uniqueness
    if user_repo::find_by_email(pool, &req.email).await?.is_some() {
        return Err(AppError::Conflict("Email already registered".into()));
    }

    // Hash password
    let password_hash = auth_service::hash_password(&req.password)?;

    // Resolve role_id: prefer explicit role_id, otherwise look up by role name
    let role_id = if let Some(rid) = req.role_id {
        Some(rid)
    } else {
        crate::repository::role_repo::find_by_name(pool, req.role.as_str())
            .await?
            .map(|r| r.id)
    };

    let user = user_repo::create_user(
        pool,
        &req.email,
        &password_hash,
        req.name.as_deref(),
        req.role.as_str(),
        UserStatus::Active.as_str(),
        false,
        role_id,
        req.department_id,
    )
    .await?;

    // Audit log
    if let Err(e) = crate::repository::audit_log_repo::create(
        pool,
        Some(auth.user_id),
        "user.create",
        Some("user"),
        Some(user.id),
        Some(serde_json::json!({
            "email": user.email,
            "role": user.role,
        })),
        None,
        None,
    )
    .await
    {
        tracing::error!("Failed to write audit log for user creation: {:?}", e);
    }

    Ok(UserResponse::from(user))
}

/// Update a user (admin+ for name, superadmin for role)
pub async fn update_user(
    pool: &PgPool,
    auth: &AuthUser,
    id: Uuid,
    req: &UpdateUserRequest,
) -> Result<UserResponse, AppError> {
    // Permission check
    auth.require_permission(pool, "users:update").await?;

    // Find target user
    let target = user_repo::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("User {} not found", id)))?;

    // Cannot edit superadmin unless you are superadmin
    if target.role == Role::SuperAdmin.as_str() && !auth.is_superadmin() {
        return Err(AppError::Forbidden(
            "Cannot edit SuperAdmin account".into(),
        ));
    }

    // Only superadmin can change roles
    if req.role.is_some() && !auth.is_superadmin() {
        return Err(AppError::Forbidden(
            "Only SuperAdmin can change user roles".into(),
        ));
    }

    // If changing role to superadmin, enforce limit
    if let Some(ref new_role) = req.role {
        if *new_role == Role::SuperAdmin && target.role != Role::SuperAdmin.as_str() {
            let superadmin_count = user_repo::count_superadmins(pool).await?;
            if superadmin_count >= 1 {
                return Err(AppError::BadRequest(
                    "SuperAdmin limit reached (maximum 1)".into(),
                ));
            }
        }
    }

    // Resolve role_id if role is being changed
    let new_role_id = if let Some(ref new_role) = req.role {
        crate::repository::role_repo::find_by_name(pool, new_role.as_str())
            .await?
            .map(|r| r.id)
    } else {
        req.role_id
    };

    let updated = user_repo::update_user(
        pool,
        id,
        req.name.as_deref(),
        req.role.as_ref().map(|r| r.as_str()),
        new_role_id,
        req.department_id.map(Some),
    )
    .await?;

    // Audit log
    if let Err(e) = crate::repository::audit_log_repo::create(
        pool,
        Some(auth.user_id),
        "user.update",
        Some("user"),
        Some(id),
        Some(serde_json::json!({
            "name_updated": req.name.is_some(),
            "role_updated": req.role.is_some(),
        })),
        None,
        None,
    )
    .await
    {
        tracing::error!("Failed to write audit log for user update: {:?}", e);
    }

    Ok(UserResponse::from(updated))
}

/// Delete a user (admin+)
pub async fn delete_user(
    pool: &PgPool,
    auth: &AuthUser,
    id: Uuid,
) -> Result<(), AppError> {
    // Permission check
    auth.require_permission(pool, "users:delete").await?;

    // Cannot delete yourself
    if auth.user_id == id {
        return Err(AppError::BadRequest("Cannot delete your own account".into()));
    }

    // Find target user
    let target = user_repo::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("User {} not found", id)))?;

    // Cannot delete superadmin unless you are superadmin
    if target.role == Role::SuperAdmin.as_str() && !auth.is_superadmin() {
        return Err(AppError::Forbidden(
            "Cannot delete SuperAdmin account".into(),
        ));
    }

    // Revoke all refresh tokens for the user before deletion
    let _ = refresh_token_repo::revoke_all_for_user(pool, id).await;

    // Audit log (before deletion so user_id is valid)
    if let Err(e) = crate::repository::audit_log_repo::create(
        pool,
        Some(auth.user_id),
        "user.delete",
        Some("user"),
        Some(id),
        Some(serde_json::json!({
            "email": target.email,
        })),
        None,
        None,
    )
    .await
    {
        tracing::error!("Failed to write audit log for user deletion: {:?}", e);
    }

    user_repo::delete_user(pool, id).await
}

/// Update a user's status (admin+)
pub async fn update_status(
    pool: &PgPool,
    auth: &AuthUser,
    id: Uuid,
    req: &UpdateStatusRequest,
) -> Result<UserResponse, AppError> {
    // Permission check
    auth.require_permission(pool, "users:manage_status").await?;

    // Cannot change your own status
    if auth.user_id == id {
        return Err(AppError::BadRequest(
            "Cannot change your own status".into(),
        ));
    }

    // Find target user
    let target = user_repo::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("User {} not found", id)))?;

    // Cannot disable superadmin unless you are superadmin
    if target.role == Role::SuperAdmin.as_str() && !auth.is_superadmin() {
        return Err(AppError::Forbidden(
            "Cannot change SuperAdmin status".into(),
        ));
    }

    let updated = user_repo::update_status(pool, id, req.status.as_str()).await?;

    // If disabling, revoke all refresh tokens
    if req.status == UserStatus::Disabled {
        let _ = refresh_token_repo::revoke_all_for_user(pool, id).await;
    }

    // Audit log
    if let Err(e) = crate::repository::audit_log_repo::create(
        pool,
        Some(auth.user_id),
        "user.status_change",
        Some("user"),
        Some(id),
        Some(serde_json::json!({
            "old_status": target.status,
            "new_status": req.status.as_str(),
        })),
        None,
        None,
    )
    .await
    {
        tracing::error!("Failed to write audit log for status change: {:?}", e);
    }

    Ok(UserResponse::from(updated))
}

/// Change the current user's password
pub async fn change_my_password(
    pool: &PgPool,
    auth: &AuthUser,
    req: &ChangeMyPasswordRequest,
) -> Result<(), AppError> {
    // Validate new password strength
    validate_password_strength(&req.new_password)
        .map_err(|e| AppError::BadRequest(e))?;

    // Find current user
    let user = user_repo::find_by_id(pool, auth.user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    // Verify current password
    if !auth_service::verify_password(&req.current_password, &user.password_hash)? {
        return Err(AppError::BadRequest("Current password is incorrect".into()));
    }

    // Hash new password
    let password_hash = auth_service::hash_password(&req.new_password)?;

    // Update password and clear must_change_password flag
    user_repo::update_password(pool, auth.user_id, &password_hash, false).await?;

    // Audit log
    if let Err(e) = crate::repository::audit_log_repo::create(
        pool,
        Some(auth.user_id),
        "user.password_change",
        Some("user"),
        Some(auth.user_id),
        None,
        None,
        None,
    )
    .await
    {
        tracing::error!("Failed to write audit log for password change: {:?}", e);
    }

    Ok(())
}

/// Reset a user's password (admin+)
pub async fn reset_password(
    pool: &PgPool,
    auth: &AuthUser,
    id: Uuid,
    req: &ResetPasswordRequest,
) -> Result<(), AppError> {
    // Permission check
    auth.require_permission(pool, "users:reset_password").await?;

    // Validate password strength
    validate_password_strength(&req.new_password)
        .map_err(|e| AppError::BadRequest(e))?;

    // Find target user
    let _target = user_repo::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("User {} not found", id)))?;

    // Hash new password
    let password_hash = auth_service::hash_password(&req.new_password)?;

    // Update password and set must_change_password = true
    user_repo::update_password(pool, id, &password_hash, true).await?;

    // Revoke all refresh tokens so the user must log in again
    let _ = refresh_token_repo::revoke_all_for_user(pool, id).await;

    // Audit log
    if let Err(e) = crate::repository::audit_log_repo::create(
        pool,
        Some(auth.user_id),
        "user.password_reset",
        Some("user"),
        Some(id),
        None,
        None,
        None,
    )
    .await
    {
        tracing::error!("Failed to write audit log for password reset: {:?}", e);
    }

    Ok(())
}
