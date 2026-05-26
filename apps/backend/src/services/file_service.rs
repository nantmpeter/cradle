use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::file::FileResponse;
use crate::repository::file_repo;

/// Ensure upload directory exists
pub async fn ensure_upload_dir(upload_dir: &str) -> Result<(), AppError> {
    let path = PathBuf::from(upload_dir);
    if !path.exists() {
        fs::create_dir_all(&path)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to create upload dir: {}", e)))?;
    }
    Ok(())
}

/// Generate a unique filename with UUID prefix
pub fn generate_filename(original: &str) -> String {
    let uuid = Uuid::new_v4();
    let ext = std::path::Path::new(original)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    if ext.is_empty() {
        uuid.to_string()
    } else {
        format!("{}.{}", uuid, ext)
    }
}

/// Save file to local storage and create DB record
pub async fn upload_file(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    original_name: &str,
    mime_type: &str,
    data: &[u8],
    upload_dir: &str,
    max_size: u64,
) -> Result<FileResponse, AppError> {
    // Check size
    if data.len() as u64 > max_size {
        return Err(AppError::PayloadTooLarge("File size exceeds limit".into()));
    }

    // Generate unique filename
    let filename = generate_filename(original_name);
    let storage_path = format!("{}/{}", upload_dir, filename);

    // Ensure directory exists
    ensure_upload_dir(upload_dir).await?;

    // Write file
    fs::write(&storage_path, data)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to write file: {}", e)))?;

    // Create DB record
    let file = file_repo::create(
        pool,
        user_id,
        &filename,
        original_name,
        mime_type,
        data.len() as i64,
        &storage_path,
        "local",
        None,
    )
    .await?;

    Ok(FileResponse::from(file))
}

/// Delete file from storage and DB
pub async fn delete_file(
    pool: &sqlx::PgPool,
    file_id: Uuid,
    user_id: Uuid,
    is_admin: bool,
) -> Result<(), AppError> {
    let file = file_repo::find_by_id(pool, file_id)
        .await?
        .ok_or_else(|| AppError::NotFound("File not found".into()))?;

    // Check ownership (unless admin)
    if !is_admin && file.user_id != user_id {
        return Err(AppError::Forbidden("Not your file".into()));
    }

    // Delete from filesystem (ignore error)
    let _ = fs::remove_file(&file.storage_path).await;

    // Delete from DB
    file_repo::delete(pool, file_id).await?;
    Ok(())
}

/// List user's files with pagination
pub async fn list_files(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    page: i64,
    per_page: i64,
) -> Result<(Vec<FileResponse>, i64), AppError> {
    let offset = (page - 1) * per_page;
    let (files, total) = file_repo::list_by_user(pool, user_id, per_page, offset).await?;
    let responses = files.into_iter().map(FileResponse::from).collect();
    Ok((responses, total))
}
