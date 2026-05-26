use sqlx::PgPool;

use crate::error::AppError;
use crate::repository::user_repo::ListParams;

/// Export users to CSV bytes with optional filters
pub async fn export_users_csv(pool: &PgPool, params: &ListParams) -> Result<Vec<u8>, AppError> {
    let users = crate::repository::user_repo::list_users_paginated(pool, params).await?;

    let mut wtr = csv::Writer::from_writer(Vec::new());
    // Header
    wtr.write_record(["id", "email", "name", "role", "status", "created_at"])
        .map_err(|e| AppError::Internal(anyhow::anyhow!("CSV write error: {}", e)))?;
    for u in &users.0 {
        wtr.write_record([
            &u.id.to_string(),
            &u.email,
            u.name.as_deref().unwrap_or(""),
            &u.role,
            &u.status,
            u.created_at.map(|d| d.to_rfc3339()).as_deref().unwrap_or(""),
        ])
        .map_err(|e| AppError::Internal(anyhow::anyhow!("CSV write error: {}", e)))?;
    }
    let bytes = wtr
        .into_inner()
        .map_err(|e| AppError::Internal(anyhow::anyhow!("CSV flush error: {}", e)))?;
    Ok(bytes)
}

/// Export users to XLSX bytes with optional filters
pub async fn export_users_xlsx(pool: &PgPool, params: &ListParams) -> Result<Vec<u8>, AppError> {
    let users = crate::repository::user_repo::list_users_paginated(pool, params).await?;

    let mut workbook = rust_xlsxwriter::Workbook::new();
    let worksheet = workbook.add_worksheet();
    let header_format = rust_xlsxwriter::Format::new().set_bold();

    let headers = ["ID", "Email", "Name", "Role", "Status", "Created At"];
    for (col, h) in headers.iter().enumerate() {
        worksheet
            .write_with_format(0, col as u16, *h, &header_format)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("XLSX error: {}", e)))?;
    }

    for (i, u) in users.0.iter().enumerate() {
        let row = (i + 1) as u32;
        worksheet.write(row, 0, u.id.to_string())
            .map_err(|e| AppError::Internal(anyhow::anyhow!("XLSX error: {}", e)))?;
        worksheet.write(row, 1, &u.email)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("XLSX error: {}", e)))?;
        worksheet.write(row, 2, u.name.as_deref().unwrap_or(""))
            .map_err(|e| AppError::Internal(anyhow::anyhow!("XLSX error: {}", e)))?;
        worksheet.write(row, 3, &u.role)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("XLSX error: {}", e)))?;
        worksheet.write(row, 4, &u.status)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("XLSX error: {}", e)))?;
        worksheet.write(row, 5, u.created_at.map(|d| d.to_rfc3339()).as_deref().unwrap_or(""))
            .map_err(|e| AppError::Internal(anyhow::anyhow!("XLSX error: {}", e)))?;
    }

    let bytes = workbook
        .save_to_buffer()
        .map_err(|e| AppError::Internal(anyhow::anyhow!("XLSX save error: {}", e)))?;
    Ok(bytes.to_vec())
}
