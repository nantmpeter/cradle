use sqlx::{PgPool, QueryBuilder, Postgres};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::notification::{Notification, NotificationResponse};

/// Create a new notification. user_id = None means broadcast.
pub async fn create(
    pool: &PgPool,
    user_id: Option<Uuid>,
    title: &str,
    content: &str,
    category: &str,
    created_by: Option<Uuid>,
) -> Result<Notification, AppError> {
    let notification = sqlx::query_as::<_, Notification>(
        r#"
        INSERT INTO notifications (user_id, title, content, category, created_by)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(title)
    .bind(content)
    .bind(category)
    .bind(created_by)
    .fetch_one(pool)
    .await?;

    Ok(notification)
}

/// Find a notification by ID
pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Notification>, AppError> {
    let notification = sqlx::query_as::<_, Notification>(
        "SELECT * FROM notifications WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(notification)
}

/// Count result helper
#[derive(sqlx::FromRow)]
struct CountResult {
    count: i64,
}

/// List notifications for a user with pagination.
/// Includes both personal notifications (user_id = $1) and broadcasts (user_id IS NULL).
/// Joins with notification_reads to determine read status.
pub async fn list_paginated(
    pool: &PgPool,
    user_id: Uuid,
    page: i64,
    per_page: i64,
    category: Option<&str>,
    is_read: Option<bool>,
) -> Result<(Vec<NotificationResponse>, i64), AppError> {
    // Data query
    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"
        SELECT n.id, n.user_id, n.title, n.content, n.category, n.is_pinned,
               n.created_by, n.created_at,
               CASE WHEN nr.notification_id IS NOT NULL THEN true ELSE false END AS is_read
        FROM notifications n
        LEFT JOIN notification_reads nr ON nr.notification_id = n.id AND nr.user_id = 
        "#,
    );
    qb.push_bind(user_id);
    qb.push(" WHERE (n.user_id = ");
    qb.push_bind(user_id);
    qb.push(" OR n.user_id IS NULL) ");

    if let Some(cat) = category {
        qb.push(" AND n.category = ");
        qb.push_bind(cat);
    }

    if let Some(read) = is_read {
        if read {
            qb.push(" AND nr.notification_id IS NOT NULL ");
        } else {
            qb.push(" AND nr.notification_id IS NULL ");
        }
    }

    qb.push(" ORDER BY n.is_pinned DESC, n.created_at DESC LIMIT ");
    qb.push_bind(per_page);
    qb.push(" OFFSET ");
    qb.push_bind((page - 1) * per_page);

    let notifications = qb
        .build_query_as::<NotificationResponse>()
        .fetch_all(pool)
        .await?;

    // Count query
    let mut cq: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"
        SELECT COUNT(*) as count
        FROM notifications n
        LEFT JOIN notification_reads nr ON nr.notification_id = n.id AND nr.user_id = 
        "#,
    );
    cq.push_bind(user_id);
    cq.push(" WHERE (n.user_id = ");
    cq.push_bind(user_id);
    cq.push(" OR n.user_id IS NULL) ");

    if let Some(cat) = category {
        cq.push(" AND n.category = ");
        cq.push_bind(cat);
    }

    if let Some(read) = is_read {
        if read {
            cq.push(" AND nr.notification_id IS NOT NULL ");
        } else {
            cq.push(" AND nr.notification_id IS NULL ");
        }
    }

    let count_result: CountResult = cq
        .build_query_as::<CountResult>()
        .fetch_one(pool)
        .await?;

    Ok((notifications, count_result.count))
}

/// Count unread notifications for a user.
/// Unread = total (personal + broadcast) - read records.
pub async fn count_unread(pool: &PgPool, user_id: Uuid) -> Result<i64, AppError> {
    let result: CountResult = sqlx::query_as::<_, CountResult>(
        r#"
        SELECT COUNT(*) as count
        FROM notifications n
        WHERE (n.user_id = $1 OR n.user_id IS NULL)
          AND NOT EXISTS (
            SELECT 1 FROM notification_reads nr
            WHERE nr.notification_id = n.id AND nr.user_id = $1
          )
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(result.count)
}

/// Mark a notification as read for a user (insert on conflict do nothing)
pub async fn insert_read(pool: &PgPool, user_id: Uuid, notification_id: Uuid) -> Result<(), AppError> {
    sqlx::query(
        r#"
        INSERT INTO notification_reads (user_id, notification_id)
        VALUES ($1, $2)
        ON CONFLICT (user_id, notification_id) DO NOTHING
        "#,
    )
    .bind(user_id)
    .bind(notification_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Mark all notifications as read for a user
pub async fn mark_all_read(pool: &PgPool, user_id: Uuid) -> Result<(), AppError> {
    sqlx::query(
        r#"
        INSERT INTO notification_reads (user_id, notification_id)
        SELECT $1, n.id
        FROM notifications n
        WHERE (n.user_id = $1 OR n.user_id IS NULL)
          AND NOT EXISTS (
            SELECT 1 FROM notification_reads nr
            WHERE nr.notification_id = n.id AND nr.user_id = $1
          )
        ON CONFLICT (user_id, notification_id) DO NOTHING
        "#,
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Delete a notification. Only the owner or the creator of a broadcast can delete.
pub async fn delete(pool: &PgPool, notification_id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM notifications WHERE id = $1")
        .bind(notification_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(crate::error::AppError::NotFound(
            "Notification not found".into(),
        ));
    }

    Ok(())
}
