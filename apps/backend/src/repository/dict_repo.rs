use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::dict::{
    CreateDictItemRequest, CreateDictTypeRequest, DictItem, DictPublicItem, DictType,
    DictTypeListParams, UpdateDictItemRequest, UpdateDictTypeRequest,
};

/// List dict types with pagination and filtering
pub async fn list_dict_types_paginated(
    pool: &PgPool,
    params: &DictTypeListParams,
) -> Result<(Vec<DictType>, i64), AppError> {
    let page = params.page.max(1);
    let per_page = params.per_page.max(1).min(100);
    let offset = (page - 1) * per_page;

    let mut where_clauses = Vec::new();
    let mut bind_idx = 1u32;

    // Dynamic query building
    let search_param_idx;
    let status_param_idx;

    if params.search.is_some() {
        where_clauses.push(format!("(dt.name ILIKE ${bind_idx} OR dt.code ILIKE ${bind_idx})"));
        search_param_idx = Some(bind_idx);
        bind_idx += 1;
    } else {
        search_param_idx = None;
    }

    if params.status.is_some() {
        where_clauses.push(format!("dt.status = ${bind_idx}"));
        status_param_idx = Some(bind_idx);
        bind_idx += 1;
    } else {
        status_param_idx = None;
    }

    let where_sql = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    // Count query
    let count_sql = format!("SELECT COUNT(*) as count FROM dict_types dt {}", where_sql);
    let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
    if let Some(idx) = search_param_idx {
        let search = format!("%{}%", params.search.as_ref().unwrap());
        count_query = count_query.bind(search);
    }
    if let Some(_idx) = status_param_idx {
        count_query = count_query.bind(&params.status);
    }
    let total = count_query.fetch_one(pool).await?;

    // Data query
    let data_sql = format!(
        "SELECT dt.* FROM dict_types dt {} ORDER BY dt.name ASC LIMIT ${bind_idx} OFFSET ${}",
        where_sql,
        bind_idx + 1
    );
    let mut data_query = sqlx::query_as::<_, DictType>(&data_sql);
    if let Some(_idx) = search_param_idx {
        let search = format!("%{}%", params.search.as_ref().unwrap());
        data_query = data_query.bind(search);
    }
    if let Some(_idx) = status_param_idx {
        data_query = data_query.bind(&params.status);
    }
    data_query = data_query.bind(per_page).bind(offset);
    let items = data_query.fetch_all(pool).await?;

    Ok((items, total))
}

/// Find a dict type by ID
pub async fn find_dict_type_by_id(pool: &PgPool, id: Uuid) -> Result<Option<DictType>, AppError> {
    let dt = sqlx::query_as::<_, DictType>("SELECT * FROM dict_types WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(dt)
}

/// Find a dict type by its unique code
pub async fn find_dict_type_by_code(pool: &PgPool, code: &str) -> Result<Option<DictType>, AppError> {
    let dt = sqlx::query_as::<_, DictType>("SELECT * FROM dict_types WHERE code = $1")
        .bind(code)
        .fetch_optional(pool)
        .await?;
    Ok(dt)
}

/// Create a new dict type
pub async fn create_dict_type(
    pool: &PgPool,
    name: &str,
    code: &str,
    status: &str,
    remark: Option<&str>,
) -> Result<DictType, AppError> {
    let dt = sqlx::query_as::<_, DictType>(
        "INSERT INTO dict_types (name, code, status, remark) VALUES ($1, $2, $3, $4) RETURNING *",
    )
    .bind(name)
    .bind(code)
    .bind(status)
    .bind(remark)
    .fetch_one(pool)
    .await?;
    Ok(dt)
}

/// Update a dict type
pub async fn update_dict_type(
    pool: &PgPool,
    id: Uuid,
    name: Option<&str>,
    status: Option<&str>,
    remark: Option<&str>,
) -> Result<DictType, AppError> {
    let dt = sqlx::query_as::<_, DictType>(
        "UPDATE dict_types SET name = COALESCE($1, name), status = COALESCE($2, status), remark = COALESCE($3, remark), updated_at = NOW() WHERE id = $4 RETURNING *",
    )
    .bind(name)
    .bind(status)
    .bind(remark)
    .bind(id)
    .fetch_one(pool)
    .await?;
    Ok(dt)
}

/// Delete a dict type (cascade deletes dict_items)
pub async fn delete_dict_type(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM dict_types WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Dict type {} not found", id)));
    }
    Ok(())
}

/// List all items belonging to a dict type
pub async fn list_items_by_type(pool: &PgPool, dict_type_id: Uuid) -> Result<Vec<DictItem>, AppError> {
    let items = sqlx::query_as::<_, DictItem>(
        "SELECT * FROM dict_items WHERE dict_type_id = $1 ORDER BY sort_order ASC, label ASC",
    )
    .bind(dict_type_id)
    .fetch_all(pool)
    .await?;
    Ok(items)
}

/// Find a dict item by ID
pub async fn find_item_by_id(pool: &PgPool, id: Uuid) -> Result<Option<DictItem>, AppError> {
    let item = sqlx::query_as::<_, DictItem>("SELECT * FROM dict_items WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(item)
}

/// Create a new dict item
pub async fn create_item(
    pool: &PgPool,
    dict_type_id: Uuid,
    req: &CreateDictItemRequest,
) -> Result<DictItem, AppError> {
    let sort_order = req.sort_order.unwrap_or(0);
    let status = req.status.as_deref().unwrap_or("active");
    let item = sqlx::query_as::<_, DictItem>(
        "INSERT INTO dict_items (dict_type_id, label, value, sort_order, status, remark) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
    )
    .bind(dict_type_id)
    .bind(&req.label)
    .bind(&req.value)
    .bind(sort_order)
    .bind(status)
    .bind(&req.remark)
    .fetch_one(pool)
    .await?;
    Ok(item)
}

/// Update a dict item
pub async fn update_item(
    pool: &PgPool,
    id: Uuid,
    req: &UpdateDictItemRequest,
) -> Result<DictItem, AppError> {
    let item = sqlx::query_as::<_, DictItem>(
        "UPDATE dict_items SET label = COALESCE($1, label), value = COALESCE($2, value), sort_order = COALESCE($3, sort_order), status = COALESCE($4, status), remark = COALESCE($5, remark), updated_at = NOW() WHERE id = $6 RETURNING *",
    )
    .bind(&req.label)
    .bind(&req.value)
    .bind(req.sort_order)
    .bind(&req.status)
    .bind(&req.remark)
    .bind(id)
    .fetch_one(pool)
    .await?;
    Ok(item)
}

/// Delete a dict item
pub async fn delete_item(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM dict_items WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Dict item {} not found", id)));
    }
    Ok(())
}

/// Find active dict items by dict type code (for public query)
pub async fn find_active_items_by_code(pool: &PgPool, code: &str) -> Result<Vec<DictPublicItem>, AppError> {
    let items = sqlx::query_as::<_, DictPublicItem>(
        r#"SELECT di.label, di.value
        FROM dict_items di
        JOIN dict_types dt ON di.dict_type_id = dt.id
        WHERE dt.code = $1 AND di.status = 'active'
        ORDER BY di.sort_order ASC, di.label ASC"#,
    )
    .bind(code)
    .fetch_all(pool)
    .await?;
    Ok(items)
}
