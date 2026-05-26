use axum::extract::{Query, State};
use axum::response::sse::{Event, Sse};
use serde::Deserialize;
use std::convert::Infallible;
use std::sync::Arc;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct SseQuery {
    pub token: Option<String>,
}

/// GET /api/sse/notifications — SSE stream for real-time notifications
/// Accepts token via query param (EventSource doesn't support custom headers)
pub async fn notification_stream(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SseQuery>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    // Parse user_id from token for filtering
    let user_id = query
        .token
        .as_deref()
        .and_then(|token| {
            let decoding_key = jsonwebtoken::DecodingKey::from_secret(state.jwt_secret.as_bytes());
            let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
            let claims: serde_json::Value =
                jsonwebtoken::decode(token, &decoding_key, &validation).ok()?.claims;
            claims.get("sub")?.as_str().and_then(|s| uuid::Uuid::parse_str(s).ok())
        })
        .unwrap_or_default();

    let receiver = state.notification_tx.subscribe();

    let stream = BroadcastStream::new(receiver).filter_map(move |msg| {
        match msg {
            Ok(notif) => {
                // Only forward notifications for this user or broadcast (user_id = None)
                if notif.user_id.is_none() || notif.user_id == Some(user_id) {
                    let data = serde_json::to_string(&notif).unwrap_or_default();
                    Some(Ok(Event::default().data(data)))
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(30))
            .text("ping"),
    )
}
