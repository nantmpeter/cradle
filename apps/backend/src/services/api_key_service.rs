use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use sha2::{Digest, Sha256};

use crate::error::AppError;
use crate::models::api_key::{
    ApiKeyListResponse, ApiKeyResponse, CreateApiKeyRequest, CreateApiKeyResponse,
};
use crate::repository::api_key_repo;
use sqlx::PgPool;
use uuid::Uuid;

/// The prefix for all generated API keys
const API_KEY_PREFIX: &str = "rk_";

/// Number of random bytes used to generate an API key (before base64 encoding)
const KEY_RANDOM_BYTES: usize = 32;

/// Create a new API key for a user.
/// Returns the plaintext key (shown only once) along with key metadata.
pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    req: &CreateApiKeyRequest,
) -> Result<CreateApiKeyResponse, AppError> {
    // Generate random key: rk_ + base64_url_safe(32 random bytes)
    let random_bytes: [u8; KEY_RANDOM_BYTES] = rand::random();
    let plaintext_key = format!("{}{}", API_KEY_PREFIX, URL_SAFE_NO_PAD.encode(random_bytes));

    // Hash the key for storage
    let key_hash = hash_api_key(&plaintext_key);

    // Extract prefix (first 8 characters) for display
    let key_prefix = plaintext_key[..8].to_string();

    // Store in database
    let api_key = api_key_repo::create(
        pool,
        &key_hash,
        &key_prefix,
        user_id,
        &req.name,
        &req.scopes,
        req.expires_at,
    )
    .await?;

    let scopes = api_key.get_scopes();
    Ok(CreateApiKeyResponse {
        id: api_key.id,
        name: api_key.name,
        key: plaintext_key,
        key_prefix: api_key.key_prefix,
        scopes,
        expires_at: api_key.expires_at,
    })
}

/// List API keys for a user (paginated).
/// Returns key metadata without the secret.
pub async fn list(
    pool: &PgPool,
    user_id: Uuid,
    page: i64,
    per_page: i64,
) -> Result<ApiKeyListResponse, AppError> {
    let page = page.max(1);
    let per_page = per_page.clamp(1, 100);

    let (keys, total) = api_key_repo::list_by_user(pool, user_id, page, per_page).await?;

    let items: Vec<ApiKeyResponse> = keys.into_iter().map(ApiKeyResponse::from).collect();

    Ok(ApiKeyListResponse {
        items,
        total,
        page,
        per_page,
    })
}

/// Revoke an API key by ID.
pub async fn revoke(pool: &PgPool, key_id: Uuid) -> Result<(), AppError> {
    // Check that the key exists
    let key = api_key_repo::find_by_id(pool, key_id)
        .await?
        .ok_or_else(|| AppError::NotFound("API key not found".into()))?;

    if key.status == "revoked" {
        return Err(AppError::BadRequest("API key is already revoked".into()));
    }

    api_key_repo::update_status(pool, key_id, "revoked").await?;
    Ok(())
}

/// Hash an API key using SHA-256
fn hash_api_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    format!("{:x}", hasher.finalize())
}
