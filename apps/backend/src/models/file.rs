use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

/// Database file record
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct File {
    pub id: Uuid,
    pub user_id: Uuid,
    pub filename: String,
    pub original_name: String,
    pub mime_type: String,
    pub size: i64,
    pub storage_path: String,
    pub storage_backend: String,
    pub thumbnail_path: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

/// Public file response (no storage_path exposure)
#[derive(Debug, Serialize)]
pub struct FileResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub filename: String,
    pub original_name: String,
    pub mime_type: String,
    pub size: i64,
    pub storage_backend: String,
    pub thumbnail_path: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

impl From<File> for FileResponse {
    fn from(f: File) -> Self {
        Self {
            id: f.id,
            user_id: f.user_id,
            filename: f.filename,
            original_name: f.original_name,
            mime_type: f.mime_type,
            size: f.size,
            storage_backend: f.storage_backend,
            thumbnail_path: f.thumbnail_path,
            created_at: f.created_at,
        }
    }
}
