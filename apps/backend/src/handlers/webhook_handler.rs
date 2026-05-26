use axum::extract::{Path, Query, State};
use axum::response::Json;
use axum::Extension;
use serde::Deserialize;
use std::sync::Arc;

use crate::error::AppError;
use crate::extractors::auth::AuthUser;
use crate::models::webhook::{
    CreateWebhookRequest, DeliveryListResponse, UpdateWebhookRequest, WebhookDeliveryResponse,
    WebhookListResponse, WebhookResponse,
};
use crate::AppState;

/// Query parameters for listing webhooks and deliveries
#[derive(Debug, Deserialize)]
pub struct ListParams {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_per_page")]
    pub per_page: i64,
}

fn default_page() -> i64 {
    1
}

fn default_per_page() -> i64 {
    20
}

/// Create a new webhook.
/// Requires `webhooks:manage` permission.
pub async fn create_webhook(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateWebhookRequest>,
) -> Result<Json<WebhookResponse>, AppError> {
    auth_user
        .require_permission(&state.db, "webhooks:manage")
        .await?;

    // Generate a random secret for HMAC signing
    let secret = generate_webhook_secret();

    let webhook =
        crate::repository::webhook_repo::create(&state.db, &req.name, &req.url, &secret, &req.events)
            .await?;

    Ok(Json(WebhookResponse::from(webhook)))
}

/// List all webhooks with pagination.
pub async fn list_webhooks(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Query(params): Query<ListParams>,
) -> Result<Json<WebhookListResponse>, AppError> {
    auth_user
        .require_permission(&state.db, "webhooks:manage")
        .await?;

    let page = params.page.max(1);
    let per_page = params.per_page.clamp(1, 100);

    let (webhooks, total) =
        crate::repository::webhook_repo::list(&state.db, page, per_page).await?;

    let items: Vec<WebhookResponse> = webhooks.into_iter().map(WebhookResponse::from).collect();

    Ok(Json(WebhookListResponse {
        items,
        total,
        page,
        per_page,
    }))
}

/// Update a webhook.
/// Requires `webhooks:manage` permission.
pub async fn update_webhook(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<uuid::Uuid>,
    Json(req): Json<UpdateWebhookRequest>,
) -> Result<Json<WebhookResponse>, AppError> {
    auth_user
        .require_permission(&state.db, "webhooks:manage")
        .await?;

    // Check webhook exists
    let _existing = crate::repository::webhook_repo::find_by_id(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Webhook not found".into()))?;

    // Validate status if provided
    if let Some(ref status) = req.status {
        if status != "active" && status != "disabled" {
            return Err(AppError::BadRequest(
                "Status must be 'active' or 'disabled'".into(),
            ));
        }
    }

    let webhook = crate::repository::webhook_repo::update(
        &state.db,
        id,
        req.name.as_deref(),
        req.url.as_deref(),
        req.events.as_deref(),
        req.status.as_deref(),
    )
    .await?;

    Ok(Json(WebhookResponse::from(webhook)))
}

/// Delete a webhook.
/// Requires `webhooks:manage` permission.
pub async fn delete_webhook(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    auth_user
        .require_permission(&state.db, "webhooks:manage")
        .await?;

    crate::repository::webhook_repo::delete(&state.db, id).await?;

    Ok(Json(serde_json::json!({
        "message": "Webhook deleted successfully"
    })))
}

/// List delivery logs for a webhook.
pub async fn list_deliveries(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<uuid::Uuid>,
    Query(params): Query<ListParams>,
) -> Result<Json<DeliveryListResponse>, AppError> {
    auth_user
        .require_permission(&state.db, "webhooks:manage")
        .await?;

    // Check webhook exists
    let _existing = crate::repository::webhook_repo::find_by_id(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Webhook not found".into()))?;

    let page = params.page.max(1);
    let per_page = params.per_page.clamp(1, 100);

    let (deliveries, total) =
        crate::repository::webhook_repo::find_deliveries_by_webhook_id(&state.db, id, page, per_page)
            .await?;

    let items: Vec<WebhookDeliveryResponse> = deliveries
        .into_iter()
        .map(WebhookDeliveryResponse::from)
        .collect();

    Ok(Json(DeliveryListResponse {
        items,
        total,
        page,
        per_page,
    }))
}

/// Generate a random webhook secret (base64-encoded 32 random bytes)
fn generate_webhook_secret() -> String {
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;
    let bytes: [u8; 32] = rand::random();
    URL_SAFE_NO_PAD.encode(bytes)
}
