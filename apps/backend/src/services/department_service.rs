use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::extractors::auth::AuthUser;
use crate::models::department::{
    CreateDepartmentRequest, DepartmentResponse, DepartmentTreeNode, UpdateDepartmentRequest,
};
use crate::repository::department_repo;

/// List all departments as flat list
pub async fn list_departments(pool: &PgPool) -> Result<Vec<DepartmentResponse>, AppError> {
    let depts = department_repo::list_all(pool).await?;
    Ok(depts.into_iter().map(|d| d.into()).collect())
}

/// Build a tree structure from a flat list of departments
pub fn build_tree(departments: Vec<DepartmentResponse>) -> Vec<DepartmentTreeNode> {
    let nodes: Vec<DepartmentTreeNode> = departments
        .into_iter()
        .map(|d| DepartmentTreeNode {
            id: d.id,
            parent_id: d.parent_id,
            name: d.name,
            code: d.code,
            sort_order: d.sort_order,
            status: d.status,
            leader: d.leader,
            children: Vec::new(),
        })
        .collect();

    // Collect parent_id -> child indices
    let mut children_map: std::collections::HashMap<Option<Uuid>, Vec<usize>> =
        std::collections::HashMap::new();
    for (i, node) in nodes.iter().enumerate() {
        children_map
            .entry(node.parent_id)
            .or_default()
            .push(i);
    }

    // We need to take nodes apart and rebuild, so use a different approach
    // Start with root nodes (parent_id = None), then recursively attach children
    fn build_children(
        parent_id: Option<Uuid>,
        nodes: &mut Vec<Option<DepartmentTreeNode>>,
        children_map: &std::collections::HashMap<Option<Uuid>, Vec<usize>>,
    ) -> Vec<DepartmentTreeNode> {
        let mut result = Vec::new();
        if let Some(indices) = children_map.get(&parent_id) {
            for &idx in indices {
                if let Some(node) = nodes[idx].take() {
                    let id = node.id;
                    let mut node = node;
                    node.children = build_children(Some(id), nodes, children_map);
                    result.push(node);
                }
            }
        }
        result.sort_by(|a, b| a.sort_order.cmp(&b.sort_order).then(a.name.cmp(&b.name)));
        result
    }

    let mut node_slots: Vec<Option<DepartmentTreeNode>> =
        nodes.into_iter().map(Some).collect();
    let tree = build_children(None, &mut node_slots, &children_map);

    tree
}

/// Create a new department
pub async fn create_department(
    pool: &PgPool,
    auth: &AuthUser,
    req: &CreateDepartmentRequest,
) -> Result<DepartmentResponse, AppError> {
    auth.require_permission(pool, "departments:create").await?;

    // Check code uniqueness
    if let Some(existing) = department_repo::find_by_code(pool, &req.code).await? {
        return Err(AppError::Conflict(format!(
            "Department with code '{}' already exists",
            existing.code
        )));
    }

    // Validate parent_id exists if provided
    if let Some(parent_id) = req.parent_id {
        if department_repo::find_by_id(pool, parent_id)
            .await?
            .is_none()
        {
            return Err(AppError::BadRequest(format!(
                "Parent department {} not found",
                parent_id
            )));
        }
    }

    let dept = department_repo::create(pool, req).await?;

    // Audit log (best-effort)
    if let Err(e) = crate::repository::audit_log_repo::create(
        pool,
        Some(auth.user_id),
        "department.create",
        Some("department"),
        Some(dept.id),
        Some(serde_json::json!({
            "name": dept.name,
            "code": dept.code,
        })),
        None,
        None,
    )
    .await
    {
        tracing::error!("Failed to write audit log for department creation: {:?}", e);
    }

    Ok(dept.into())
}

/// Update a department
pub async fn update_department(
    pool: &PgPool,
    auth: &AuthUser,
    id: Uuid,
    req: &UpdateDepartmentRequest,
) -> Result<DepartmentResponse, AppError> {
    auth.require_permission(pool, "departments:update").await?;

    // Check department exists
    if department_repo::find_by_id(pool, id).await?.is_none() {
        return Err(AppError::NotFound(format!("Department {} not found", id)));
    }

    let dept = department_repo::update(pool, id, req).await?;

    // Audit log (best-effort)
    if let Err(e) = crate::repository::audit_log_repo::create(
        pool,
        Some(auth.user_id),
        "department.update",
        Some("department"),
        Some(id),
        Some(serde_json::json!({
            "name": req.name,
        })),
        None,
        None,
    )
    .await
    {
        tracing::error!("Failed to write audit log for department update: {:?}", e);
    }

    Ok(dept.into())
}

/// Delete a department (block if users exist)
pub async fn delete_department(
    pool: &PgPool,
    auth: &AuthUser,
    id: Uuid,
) -> Result<(), AppError> {
    auth.require_permission(pool, "departments:delete").await?;

    // Check department exists
    let existing = department_repo::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Department {} not found", id)))?;

    // Block if department has users
    let user_count = department_repo::count_users_by_department(pool, id).await?;
    if user_count > 0 {
        return Err(AppError::Conflict(format!(
            "Cannot delete department: {} user(s) are still in this department. Please transfer users first.",
            user_count
        )));
    }

    department_repo::delete(pool, id).await?;

    // Audit log (best-effort)
    if let Err(e) = crate::repository::audit_log_repo::create(
        pool,
        Some(auth.user_id),
        "department.delete",
        Some("department"),
        Some(id),
        Some(serde_json::json!({
            "name": existing.name,
            "code": existing.code,
        })),
        None,
        None,
    )
    .await
    {
        tracing::error!("Failed to write audit log for department deletion: {:?}", e);
    }

    Ok(())
}
