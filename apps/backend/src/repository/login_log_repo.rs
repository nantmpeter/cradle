use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::login_log::{CreateLoginLogParams, LoginLog, LoginLogListParams};

/// Insert a new login log record
pub async fn insert_log(pool: &PgPool, params: &CreateLoginLogParams) -> Result<LoginLog, AppError> {
    let log = sqlx::query_as::<_, LoginLog>(
        "INSERT INTO login_logs (user_id, email, event, ip_address, user_agent, os, browser, success, fail_reason) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING *",
    )
    .bind(params.user_id)
    .bind(&params.email)
    .bind(&params.event)
    .bind(&params.ip_address)
    .bind(&params.user_agent)
    .bind(&params.os)
    .bind(&params.browser)
    .bind(params.success)
    .bind(&params.fail_reason)
    .fetch_one(pool)
    .await?;
    Ok(log)
}

/// List login logs with pagination and filtering
pub async fn list_logs_paginated(
    pool: &PgPool,
    params: &LoginLogListParams,
) -> Result<(Vec<LoginLog>, i64), AppError> {
    let page = params.page.max(1);
    let per_page = params.per_page.max(1).min(100);
    let offset = (page - 1) * per_page;

    let mut where_clauses = Vec::new();
    let mut bind_idx = 1u32;

    let email_param_idx;
    let event_param_idx;
    let ip_param_idx;
    let from_param_idx;
    let to_param_idx;

    if params.email.is_some() {
        where_clauses.push(format!("ll.email ILIKE ${bind_idx}"));
        email_param_idx = Some(bind_idx);
        bind_idx += 1;
    } else {
        email_param_idx = None;
    }

    if params.event.is_some() {
        where_clauses.push(format!("ll.event = ${bind_idx}"));
        event_param_idx = Some(bind_idx);
        bind_idx += 1;
    } else {
        event_param_idx = None;
    }

    if params.ip_address.is_some() {
        where_clauses.push(format!("ll.ip_address = ${bind_idx}"));
        ip_param_idx = Some(bind_idx);
        bind_idx += 1;
    } else {
        ip_param_idx = None;
    }

    if params.start_date.is_some() {
        where_clauses.push(format!("ll.login_at >= ${bind_idx}::timestamptz"));
        from_param_idx = Some(bind_idx);
        bind_idx += 1;
    } else {
        from_param_idx = None;
    }

    if params.end_date.is_some() {
        where_clauses.push(format!("ll.login_at <= ${bind_idx}::timestamptz"));
        to_param_idx = Some(bind_idx);
        bind_idx += 1;
    } else {
        to_param_idx = None;
    }

    let where_sql = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    // Count
    let count_sql = format!("SELECT COUNT(*) as count FROM login_logs ll {}", where_sql);
    let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
    if let Some(_idx) = email_param_idx {
        let email = format!("%{}%", params.email.as_ref().unwrap());
        count_query = count_query.bind(email);
    }
    if let Some(_idx) = event_param_idx {
        count_query = count_query.bind(&params.event);
    }
    if let Some(_idx) = ip_param_idx {
        count_query = count_query.bind(&params.ip_address);
    }
    if let Some(_idx) = from_param_idx {
        count_query = count_query.bind(params.start_date.as_ref().unwrap());
    }
    if let Some(_idx) = to_param_idx {
        count_query = count_query.bind(params.end_date.as_ref().unwrap());
    }
    let total = count_query.fetch_one(pool).await?;

    // Data
    let data_sql = format!(
        "SELECT ll.* FROM login_logs ll {} ORDER BY ll.login_at DESC LIMIT ${bind_idx} OFFSET ${}",
        where_sql,
        bind_idx + 1
    );
    let mut data_query = sqlx::query_as::<_, LoginLog>(&data_sql);
    if let Some(_idx) = email_param_idx {
        let email = format!("%{}%", params.email.as_ref().unwrap());
        data_query = data_query.bind(email);
    }
    if let Some(_idx) = event_param_idx {
        data_query = data_query.bind(&params.event);
    }
    if let Some(_idx) = ip_param_idx {
        data_query = data_query.bind(&params.ip_address);
    }
    if let Some(_idx) = from_param_idx {
        data_query = data_query.bind(params.start_date.as_ref().unwrap());
    }
    if let Some(_idx) = to_param_idx {
        data_query = data_query.bind(params.end_date.as_ref().unwrap());
    }
    data_query = data_query.bind(per_page).bind(offset);
    let logs = data_query.fetch_all(pool).await?;

    Ok((logs, total))
}

/// Find a login log by ID
pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<LoginLog>, AppError> {
    let log = sqlx::query_as::<_, LoginLog>("SELECT * FROM login_logs WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(log)
}
