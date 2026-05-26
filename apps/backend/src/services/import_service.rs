use calamine::{open_workbook_auto_from_rs, Data, Range, Reader};
use serde::Serialize;
use sqlx::PgPool;
use std::io::Cursor;

use crate::error::AppError;
use crate::extractors::auth::AuthUser;
use crate::services::auth_service;

#[derive(Debug, Serialize)]
pub struct ImportResult {
    pub success_count: u32,
    pub fail_count: u32,
    pub errors: Vec<RowError>,
    pub generated_passwords: Vec<GeneratedPassword>,
}

#[derive(Debug, Serialize)]
pub struct RowError {
    pub row: u32,
    pub email: String,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct GeneratedPassword {
    pub email: String,
    pub password: String,
}

/// Extract a string value from a cell in a Range
fn cell_string(range: &Range<Data>, row: usize, col: usize) -> Option<String> {
    range
        .get((row, col))
        .and_then(|cell| {
            match cell {
                Data::String(s) => Some(s.clone()),
                Data::Float(f) => Some(if *f == (*f as i64) as f64 {
                    (*f as i64).to_string()
                } else {
                    f.to_string()
                }),
                Data::Int(i) => Some(i.to_string()),
                _ => None,
            }
        })
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Import users from xlsx bytes
pub async fn import_users(
    pool: &PgPool,
    auth: &AuthUser,
    xlsx_bytes: &[u8],
) -> Result<ImportResult, AppError> {
    auth.require_permission(pool, "users:import").await?;

    let cursor = Cursor::new(xlsx_bytes.to_vec());
    let mut workbook = open_workbook_auto_from_rs(cursor)
        .map_err(|e| AppError::BadRequest(format!("Failed to parse xlsx file: {}", e)))?;

    let sheet_names = workbook.sheet_names().to_vec();
    let sheet_name = sheet_names
        .first()
        .ok_or_else(|| AppError::BadRequest("Excel file has no sheets".into()))?
        .clone();

    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|e| AppError::BadRequest(format!("Failed to read sheet '{}': {}", sheet_name, e)))?;

    let mut result = ImportResult {
        success_count: 0,
        fail_count: 0,
        errors: Vec::new(),
        generated_passwords: Vec::new(),
    };

    // Count rows (skip header at row 0)
    let row_count = range.rows().count();
    if row_count <= 1 {
        return Ok(result);
    }

    for row_idx in 1..row_count {
        let row_num = (row_idx + 1) as u32; // 1-indexed for display

        // Extract fields from cells
        let email = cell_string(&range, row_idx, 0).unwrap_or_default();
        let name = cell_string(&range, row_idx, 1).unwrap_or_default();
        let role = cell_string(&range, row_idx, 2).unwrap_or_else(|| "user".to_string());
        let dept_code = cell_string(&range, row_idx, 3);

        // Validate email
        if email.is_empty() {
            result.errors.push(RowError {
                row: row_num,
                email: String::new(),
                reason: "Email is required".into(),
            });
            result.fail_count += 1;
            continue;
        }

        // Validate email format
        if !email.contains('@') || !email.contains('.') {
            result.errors.push(RowError {
                row: row_num,
                email: email.clone(),
                reason: "Invalid email format".into(),
            });
            result.fail_count += 1;
            continue;
        }

        // Check email uniqueness
        if let Ok(Some(_)) = crate::repository::user_repo::find_by_email(pool, &email).await {
            result.errors.push(RowError {
                row: row_num,
                email: email.clone(),
                reason: "Email already exists".into(),
            });
            result.fail_count += 1;
            continue;
        }

        // Validate name — default from email
        let user_name = if name.is_empty() {
            email.split('@').next().unwrap_or("User").to_string()
        } else {
            name
        };

        // Resolve role
        let role_str = match role.to_lowercase().as_str() {
            "admin" => "admin",
            _ => "user",
        };

        // Find role_id
        let role_id = match crate::repository::role_repo::find_by_name(pool, role_str).await {
            Ok(Some(r)) => Some(r.id),
            _ => None,
        };

        // Resolve department_id
        let department_id = match dept_code {
            Some(ref code) => {
                match crate::repository::department_repo::find_by_code(pool, code).await {
                    Ok(Some(dept)) => Some(dept.id),
                    _ => {
                        result.errors.push(RowError {
                            row: row_num,
                            email: email.clone(),
                            reason: format!("Department code '{}' not found", code),
                        });
                        result.fail_count += 1;
                        continue;
                    }
                }
            }
            None => None,
        };

        // Generate random password
        let password = generate_random_password();
        let password_hash = match auth_service::hash_password(&password) {
            Ok(h) => h,
            Err(e) => {
                result.errors.push(RowError {
                    row: row_num,
                    email: email.clone(),
                    reason: format!("Failed to hash password: {}", e),
                });
                result.fail_count += 1;
                continue;
            }
        };

        // Create user
        let create_result = crate::repository::user_repo::create_user(
            pool,
            &email,
            &password_hash,
            Some(&user_name),
            role_str,
            "active",
            true, // must_change_password
            role_id,
            department_id,
        )
        .await;

        match create_result {
            Ok(_) => {
                result.success_count += 1;
                result.generated_passwords.push(GeneratedPassword {
                    email: email.clone(),
                    password,
                });
            }
            Err(e) => {
                result.errors.push(RowError {
                    row: row_num,
                    email: email.clone(),
                    reason: format!("Failed to create user: {}", e),
                });
                result.fail_count += 1;
            }
        }
    }

    // Audit log (best-effort)
    if let Err(e) = crate::repository::audit_log_repo::create(
        pool,
        Some(auth.user_id),
        "users.import",
        Some("user"),
        None,
        Some(serde_json::json!({
            "success_count": result.success_count,
            "fail_count": result.fail_count,
        })),
        None,
        None,
    )
    .await
    {
        tracing::error!("Failed to write audit log for user import: {:?}", e);
    }

    Ok(result)
}

/// Generate an empty import template (xlsx)
pub fn generate_import_template() -> Vec<u8> {
    use rust_xlsxwriter::Workbook;

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    // Header row
    worksheet.write(0, 0, "Email").unwrap();
    worksheet.write(0, 1, "Name").unwrap();
    worksheet.write(0, 2, "Role (admin/user)").unwrap();
    worksheet.write(0, 3, "Department Code").unwrap();

    // Example row
    worksheet.write(1, 0, "user@example.com").unwrap();
    worksheet.write(1, 1, "John Doe").unwrap();
    worksheet.write(1, 2, "user").unwrap();
    worksheet.write(1, 3, "").unwrap();

    workbook.save_to_buffer().unwrap_or_default()
}

/// Generate a random 16-character alphanumeric password
fn generate_random_password() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789!@#$%^&*";
    let mut rng = rand::rng();
    (0..16)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
