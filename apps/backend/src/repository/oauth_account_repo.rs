use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::oauth_account::OAuthAccount;

/// Create a new OAuth account link
pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    provider: &str,
    provider_user_id: &str,
    union_id: Option<&str>,
) -> Result<OAuthAccount, AppError> {
    let account = sqlx::query_as::<_, OAuthAccount>(
        r#"INSERT INTO oauth_accounts (user_id, provider, provider_user_id, union_id)
           VALUES ($1, $2, $3, $4)
           RETURNING *"#,
    )
    .bind(user_id)
    .bind(provider)
    .bind(provider_user_id)
    .bind(union_id)
    .fetch_one(pool)
    .await?;

    Ok(account)
}

/// Find an OAuth account by provider and provider user ID
pub async fn find_by_provider_user(
    pool: &PgPool,
    provider: &str,
    provider_user_id: &str,
) -> Result<Option<OAuthAccount>, AppError> {
    let account = sqlx::query_as::<_, OAuthAccount>(
        "SELECT * FROM oauth_accounts WHERE provider = $1 AND provider_user_id = $2",
    )
    .bind(provider)
    .bind(provider_user_id)
    .fetch_optional(pool)
    .await?;

    Ok(account)
}

/// Find all OAuth accounts for a user
pub async fn find_by_user_id(pool: &PgPool, user_id: Uuid) -> Result<Vec<OAuthAccount>, AppError> {
    let accounts = sqlx::query_as::<_, OAuthAccount>(
        "SELECT * FROM oauth_accounts WHERE user_id = $1 ORDER BY created_at",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(accounts)
}

/// Find an OAuth account by user ID and provider
pub async fn find_by_user_and_provider(
    pool: &PgPool,
    user_id: Uuid,
    provider: &str,
) -> Result<Option<OAuthAccount>, AppError> {
    let account = sqlx::query_as::<_, OAuthAccount>(
        "SELECT * FROM oauth_accounts WHERE user_id = $1 AND provider = $2",
    )
    .bind(user_id)
    .bind(provider)
    .fetch_optional(pool)
    .await?;

    Ok(account)
}

/// Delete an OAuth account binding
pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
    sqlx::query("DELETE FROM oauth_accounts WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
