use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::file::File;

pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    filename: &str,
    original_name: &str,
    mime_type: &str,
    size: i64,
    storage_path: &str,
    storage_backend: &str,
    thumbnail_path: Option<&str>,
) -> Result<File, AppError> {
    let file = sqlx::query_as::<_, File>(
        "INSERT INTO files (user_id, filename, original_name, mime_type, size, storage_path, storage_backend, thumbnail_path)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *",
    )
    .bind(user_id)
    .bind(filename)
    .bind(original_name)
    .bind(mime_type)
    .bind(size)
    .bind(storage_path)
    .bind(storage_backend)
    .bind(thumbnail_path)
    .fetch_one(pool)
    .await?;
    Ok(file)
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<File>, AppError> {
    let file = sqlx::query_as::<_, File>("SELECT * FROM files WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(file)
}

pub async fn list_by_user(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<(Vec<File>, i64), AppError> {
    let files = sqlx::query_as::<_, File>(
        "SELECT * FROM files WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    #[derive(sqlx::FromRow)]
    struct CountResult { count: i64 }
    let count: CountResult = sqlx::query_as::<_, CountResult>(
        "SELECT COUNT(*) as count FROM files WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok((files, count.count))
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM files WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("File {} not found", id)));
    }
    Ok(())
}
