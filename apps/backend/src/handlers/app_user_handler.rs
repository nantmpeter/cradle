use axum::extract::State;
use axum::response::Json;
use axum::Extension;
use std::sync::Arc;

use crate::error::AppError;
use crate::extractors::auth::AuthUser;
use crate::models::oauth_account::OAuthAccountResponse;
use crate::AppState;

/// GET /api/v1/app/auth/bindings
/// List all social account bindings for the authenticated user.
pub async fn list_bindings(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<Vec<OAuthAccountResponse>>, AppError> {
    let accounts =
        crate::repository::oauth_account_repo::find_by_user_id(&state.db, auth_user.user_id)
            .await?;

    let responses: Vec<OAuthAccountResponse> =
        accounts.into_iter().map(OAuthAccountResponse::from).collect();

    Ok(Json(responses))
}
