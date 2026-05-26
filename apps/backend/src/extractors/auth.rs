use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::AppState;
use crate::error::AppError;

/// The type of client making the request.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ClientType {
    Admin,
    App,
}

impl Default for ClientType {
    fn default() -> Self {
        ClientType::Admin
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthUser {
    pub user_id: uuid::Uuid,
    pub email: String,
    pub role: String,
    pub role_id: Option<uuid::Uuid>,
    pub department_id: Option<uuid::Uuid>,
    pub jti: uuid::Uuid,
    #[serde(default)]
    pub client_type: ClientType,
}

impl AuthUser {
    /// Check if the user has admin or superadmin role
    pub fn is_admin(&self) -> bool {
        self.role == "admin" || self.role == "superadmin"
    }

    /// Check if the user has superadmin role
    pub fn is_superadmin(&self) -> bool {
        self.role == "superadmin"
    }

    /// Require admin role (admin or superadmin). Returns 403 if not.
    pub fn require_admin(&self) -> Result<&Self, AppError> {
        if self.is_admin() {
            Ok(self)
        } else {
            Err(AppError::Forbidden(
                "Admin access required".into(),
            ))
        }
    }

    /// Require superadmin role. Returns 403 if not.
    pub fn require_superadmin(&self) -> Result<&Self, AppError> {
        if self.is_superadmin() {
            Ok(self)
        } else {
            Err(AppError::Forbidden(
                "SuperAdmin access required".into(),
            ))
        }
    }

    /// Require a specific permission string.
    /// Superadmin fast-path: skip DB query.
    /// Other roles: query role_permissions JOIN permissions.
    pub async fn require_permission(
        &self,
        pool: &PgPool,
        permission: &str,
    ) -> Result<&Self, AppError> {
        // Superadmin fast path
        if self.is_superadmin() {
            return Ok(self);
        }

        // Query user permissions from DB
        let user_permissions = crate::repository::permission_repo::find_by_user_id(pool, self.user_id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to query permissions: {:?}", e);
                AppError::Internal(anyhow::anyhow!("Failed to query permissions"))
            })?;

        if user_permissions.iter().any(|p| p == permission) {
            Ok(self)
        } else {
            Err(AppError::Forbidden(format!(
                "Permission denied: {}",
                permission
            )))
        }
    }

    /// Require all of the specified permissions
    pub async fn require_permissions(
        &self,
        pool: &PgPool,
        permissions: &[&str],
    ) -> Result<&Self, AppError> {
        // Superadmin fast path
        if self.is_superadmin() {
            return Ok(self);
        }

        let user_permissions = crate::repository::permission_repo::find_by_user_id(pool, self.user_id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to query permissions: {:?}", e);
                AppError::Internal(anyhow::anyhow!("Failed to query permissions"))
            })?;

        for perm in permissions {
            if !user_permissions.iter().any(|p| p == *perm) {
                return Err(AppError::Forbidden(format!(
                    "Permission denied: {}",
                    perm
                )));
            }
        }

        Ok(self)
    }

    /// Get the data_scope of the user's role
    async fn get_data_scope(&self, pool: &PgPool) -> String {
        if self.is_superadmin() {
            return "all".to_string();
        }
        if let Some(role_id) = self.role_id {
            if let Ok(Some(role)) = crate::repository::role_repo::find_by_id(pool, role_id).await {
                return role.data_scope;
            }
        }
        "self".to_string()
    }

    /// Get department IDs visible to this user based on data_scope.
    /// Returns None if all data is visible (data_scope = "all").
    /// Returns Some(vec![]) if only own data (data_scope = "self" without department).
    /// Returns Some(dept_ids) for department-scoped access.
    pub async fn get_visible_department_ids(&self, pool: &PgPool) -> Option<Vec<uuid::Uuid>> {
        let scope = self.get_data_scope(pool).await;

        match scope.as_str() {
            "all" => None, // No restriction
            "self" => {
                // Only see users in own department (or just self if no department)
                if let Some(dept_id) = self.department_id {
                    Some(vec![dept_id])
                } else {
                    Some(vec![]) // No department = only self
                }
            }
            "department" => {
                if let Some(dept_id) = self.department_id {
                    Some(vec![dept_id])
                } else {
                    Some(vec![])
                }
            }
            "department_and_sub" => {
                if let Some(dept_id) = self.department_id {
                    // Get department and all sub-departments
                    match crate::repository::department_repo::list_all(pool).await {
                        Ok(all_depts) => {
                            let mut visible = vec![dept_id];
                            collect_sub_departments(&all_depts, dept_id, &mut visible);
                            Some(visible)
                        }
                        Err(_) => Some(vec![dept_id]),
                    }
                } else {
                    Some(vec![])
                }
            }
            _ => None,
        }
    }
}

/// Recursively collect sub-department IDs
fn collect_sub_departments(
    all_depts: &[crate::models::department::Department],
    parent_id: uuid::Uuid,
    result: &mut Vec<uuid::Uuid>,
) {
    for dept in all_depts {
        if dept.parent_id == Some(parent_id) {
            result.push(dept.id);
            collect_sub_departments(all_depts, dept.id, result);
        }
    }
}

impl FromRequestParts<std::sync::Arc<AppState>> for AuthUser {
    type Rejection = AppError;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &std::sync::Arc<AppState>,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        let result = parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .ok_or_else(|| AppError::Unauthorized("Not authenticated".into()));

        std::future::ready(result)
    }
}
