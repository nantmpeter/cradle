use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::department::{CreateDepartmentRequest, Department, UpdateDepartmentRequest};

/// Find a department by ID
pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Department>, AppError> {
    let dept = sqlx::query_as::<_, Department>("SELECT * FROM departments WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(dept)
}

/// List all departments ordered by sort_order, name
pub async fn list_all(pool: &PgPool) -> Result<Vec<Department>, AppError> {
    let depts = sqlx::query_as::<_, Department>(
        "SELECT * FROM departments ORDER BY sort_order ASC, name ASC",
    )
    .fetch_all(pool)
    .await?;
    Ok(depts)
}

/// Find a department by its unique code
pub async fn find_by_code(pool: &PgPool, code: &str) -> Result<Option<Department>, AppError> {
    let dept = sqlx::query_as::<_, Department>("SELECT * FROM departments WHERE code = $1")
        .bind(code)
        .fetch_optional(pool)
        .await?;
    Ok(dept)
}

/// Create a new department
pub async fn create(pool: &PgPool, req: &CreateDepartmentRequest) -> Result<Department, AppError> {
    let status = req.status.as_deref().unwrap_or("active");
    let sort_order = req.sort_order.unwrap_or(0);
    let dept = sqlx::query_as::<_, Department>(
        "INSERT INTO departments (parent_id, name, code, sort_order, status, leader) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
    )
    .bind(req.parent_id)
    .bind(&req.name)
    .bind(&req.code)
    .bind(sort_order)
    .bind(status)
    .bind(&req.leader)
    .fetch_one(pool)
    .await?;
    Ok(dept)
}

/// Update a department
pub async fn update(pool: &PgPool, id: Uuid, req: &UpdateDepartmentRequest) -> Result<Department, AppError> {
    let dept = sqlx::query_as::<_, Department>(
        "UPDATE departments SET name = COALESCE($1, name), sort_order = COALESCE($2, sort_order), status = COALESCE($3, status), leader = COALESCE($4, leader), updated_at = NOW() WHERE id = $5 RETURNING *",
    )
    .bind(&req.name)
    .bind(req.sort_order)
    .bind(&req.status)
    .bind(&req.leader)
    .bind(id)
    .fetch_one(pool)
    .await?;
    Ok(dept)
}

/// Delete a department by ID
pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM departments WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Department {} not found", id)));
    }
    Ok(())
}

/// Count users assigned to a specific department
pub async fn count_users_by_department(pool: &PgPool, dept_id: Uuid) -> Result<i64, AppError> {
    #[derive(sqlx::FromRow)]
    struct CountResult {
        count: i64,
    }
    let result: CountResult =
        sqlx::query_as::<_, CountResult>("SELECT COUNT(*) as count FROM users WHERE department_id = $1")
            .bind(dept_id)
            .fetch_one(pool)
            .await?;
    Ok(result.count)
}
