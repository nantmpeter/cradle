use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::audit_log::{AuditLog, AuditLogResponse};

/// Create a new audit log entry
pub async fn create(
    pool: &PgPool,
    user_id: Option<Uuid>,
    action: &str,
    resource_type: Option<&str>,
    resource_id: Option<Uuid>,
    details: Option<serde_json::Value>,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        INSERT INTO audit_logs (user_id, action, resource_type, resource_id, details, ip_address, user_agent)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(user_id)
    .bind(action)
    .bind(resource_type)
    .bind(resource_id)
    .bind(details)
    .bind(ip_address)
    .bind(user_agent)
    .execute(pool)
    .await?;

    Ok(())
}

/// Find an audit log by ID (returns DB model)
pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<AuditLog>, AppError> {
    let log = sqlx::query_as::<_, AuditLog>("SELECT * FROM audit_logs WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(log)
}

/// Helper struct for count query
#[derive(sqlx::FromRow)]
struct CountResult {
    count: i64,
}

/// Query parameters for paginated audit log listing
pub struct AuditLogQueryParams {
    pub page: i64,
    pub per_page: i64,
    pub user_id: Option<Uuid>,
    pub action: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub ip_address: Option<String>,
}

/// List audit logs with pagination and filtering, returns response with user_email joined
pub async fn list_paginated(
    pool: &PgPool,
    params: &AuditLogQueryParams,
) -> Result<(Vec<AuditLogResponse>, i64), AppError> {
    // Build data query
    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"SELECT al.id, al.user_id, u.email AS user_email, al.action,
           al.resource_type, al.resource_id, al.details, al.ip_address,
           al.user_agent, al.created_at
           FROM audit_logs al
           LEFT JOIN users u ON al.user_id = u.id
           WHERE 1=1"#,
    );

    apply_filters(&mut qb, params);

    qb.push(" ORDER BY al.created_at DESC LIMIT ");
    qb.push_bind(params.per_page);
    qb.push(" OFFSET ");
    qb.push_bind((params.page - 1) * params.per_page);

    let logs = qb.build_query_as::<AuditLogResponse>().fetch_all(pool).await?;

    // Build count query (use "al" alias to match apply_filters)
    let mut cq: QueryBuilder<Postgres> =
        QueryBuilder::new("SELECT COUNT(*) as count FROM audit_logs al WHERE 1=1");
    apply_filters(&mut cq, params);

    let count_result: CountResult = cq.build_query_as::<CountResult>().fetch_one(pool).await?;

    Ok((logs, count_result.count))
}

/// Apply filter conditions to a QueryBuilder
fn apply_filters(qb: &mut QueryBuilder<Postgres>, params: &AuditLogQueryParams) {
    if let Some(uid) = params.user_id {
        qb.push(" AND al.user_id = ");
        qb.push_bind(uid);
    }
    if let Some(ref action) = params.action {
        let pattern = format!("%{}%", action);
        qb.push(" AND al.action ILIKE ");
        qb.push_bind(pattern);
    }
    if let Some(ref from) = params.from {
        qb.push(" AND al.created_at >= ");
        qb.push_bind(from.clone());
    }
    if let Some(ref to) = params.to {
        qb.push(" AND al.created_at <= ");
        qb.push_bind(to.clone());
    }
    if let Some(ref resource_type) = params.resource_type {
        qb.push(" AND al.resource_type = ");
        qb.push_bind(resource_type.clone());
    }
    if let Some(resource_id) = params.resource_id {
        qb.push(" AND al.resource_id = ");
        qb.push_bind(resource_id);
    }
    if let Some(ref ip_address) = params.ip_address {
        let pattern = format!("%{}%", ip_address);
        qb.push(" AND al.ip_address ILIKE ");
        qb.push_bind(pattern);
    }
}

/// Find an audit log response by ID with user_email joined
pub async fn find_response_by_id(pool: &PgPool, id: Uuid) -> Result<Option<AuditLogResponse>, AppError> {
    let log = sqlx::query_as::<_, AuditLogResponse>(
        r#"
        SELECT al.id, al.user_id, u.email AS user_email, al.action,
               al.resource_type, al.resource_id, al.details, al.ip_address,
               al.user_agent, al.created_at
        FROM audit_logs al
        LEFT JOIN users u ON al.user_id = u.id
        WHERE al.id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(log)
}

/// Export all audit logs matching filter params (no pagination)
pub async fn export_filtered(
    pool: &PgPool,
    params: &AuditLogQueryParams,
) -> Result<Vec<AuditLogResponse>, AppError> {
    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"SELECT al.id, al.user_id, u.email AS user_email, al.action,
           al.resource_type, al.resource_id, al.details, al.ip_address,
           al.user_agent, al.created_at
           FROM audit_logs al
           LEFT JOIN users u ON al.user_id = u.id
           WHERE 1=1"#,
    );

    apply_filters(&mut qb, params);
    qb.push(" ORDER BY al.created_at DESC");

    let logs = qb
        .build_query_as::<AuditLogResponse>()
        .fetch_all(pool)
        .await?;

    Ok(logs)
}
