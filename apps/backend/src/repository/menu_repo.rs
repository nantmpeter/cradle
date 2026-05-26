use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::menu::{CreateMenuRequest, Menu, UpdateMenuRequest};

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Menu>, AppError> {
    let menu = sqlx::query_as::<_, Menu>("SELECT * FROM menus WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(menu)
}

pub async fn list_all(pool: &PgPool) -> Result<Vec<Menu>, AppError> {
    let menus = sqlx::query_as::<_, Menu>(
        "SELECT * FROM menus ORDER BY sort_order ASC, created_at ASC",
    )
    .fetch_all(pool)
    .await?;
    Ok(menus)
}

/// List active menus filtered by user permission names (or menus with no permission requirement)
pub async fn list_by_permission_names(
    pool: &PgPool,
    permission_names: &[String],
) -> Result<Vec<Menu>, AppError> {
    let menus = sqlx::query_as::<_, Menu>(
        "SELECT m.* FROM menus m
         LEFT JOIN permissions p ON m.permission_id = p.id
         WHERE m.status = 'active'
           AND (m.permission_id IS NULL OR p.name = ANY($1))
         ORDER BY m.sort_order ASC, m.created_at ASC",
    )
    .bind(permission_names)
    .fetch_all(pool)
    .await?;
    Ok(menus)
}

pub async fn create(pool: &PgPool, req: &CreateMenuRequest) -> Result<Menu, AppError> {
    let menu = sqlx::query_as::<_, Menu>(
        "INSERT INTO menus (parent_id, title_key, title_label, path, icon, sort_order, permission_id)
         VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *",
    )
    .bind(req.parent_id)
    .bind(&req.title_key)
    .bind(&req.title_label)
    .bind(&req.path)
    .bind(&req.icon)
    .bind(req.sort_order)
    .bind(req.permission_id)
    .fetch_one(pool)
    .await?;
    Ok(menu)
}

pub async fn update(pool: &PgPool, id: Uuid, req: &UpdateMenuRequest) -> Result<Menu, AppError> {
    let menu = sqlx::query_as::<_, Menu>(
        "UPDATE menus SET
            parent_id = COALESCE($1, parent_id),
            title_key = COALESCE($2, title_key),
            title_label = COALESCE($3, title_label),
            path = COALESCE($4, path),
            icon = COALESCE($5, icon),
            sort_order = COALESCE($6, sort_order),
            permission_id = COALESCE($7, permission_id),
            status = COALESCE($8, status),
            updated_at = NOW()
         WHERE id = $9 RETURNING *",
    )
    .bind(req.parent_id)
    .bind(&req.title_key)
    .bind(&req.title_label)
    .bind(&req.path)
    .bind(&req.icon)
    .bind(req.sort_order)
    .bind(req.permission_id)
    .bind(&req.status)
    .bind(id)
    .fetch_one(pool)
    .await?;
    Ok(menu)
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM menus WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Menu {} not found", id)));
    }
    Ok(())
}
