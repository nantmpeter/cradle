use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::error::AppError;
use crate::extractors::auth::AuthUser;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    5
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub results: Vec<SearchItem>,
}

#[derive(Debug, Serialize)]
pub struct SearchItem {
    pub r#type: String,  // "user" | "role" | "menu" | "config"
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub path: String,
}

/// GET /api/search?q=... — Global search
pub async fn search(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Query(query): Query<SearchQuery>,
) -> Result<Json<SearchResult>, AppError> {
    let pattern = format!("%{}%", query.q);
    let limit = query.limit.clamp(1, 20);
    let mut results = Vec::new();

    // Search users
    let users: Vec<(uuid::Uuid, String, Option<String>)> = sqlx::query_as(
        "SELECT id, email, name FROM users WHERE email ILIKE $1 OR name ILIKE $1 LIMIT $2",
    )
    .bind(&pattern)
    .bind(limit)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    for (id, email, name) in users {
        results.push(SearchItem {
            r#type: "user".into(),
            id: id.to_string(),
            title: name.clone().unwrap_or_else(|| email.clone()),
            subtitle: Some(email),
            path: "/dashboard/users".into(),
        });
    }

    // Search roles
    let roles: Vec<(uuid::Uuid, String, Option<String>)> = sqlx::query_as(
        "SELECT id, name, description FROM roles WHERE name ILIKE $1 OR description ILIKE $1 LIMIT $2",
    )
    .bind(&pattern)
    .bind(limit)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    for (id, name, desc) in roles {
        results.push(SearchItem {
            r#type: "role".into(),
            id: id.to_string(),
            title: name,
            subtitle: desc,
            path: "/dashboard/roles".into(),
        });
    }

    // Search menus
    let menus: Vec<(uuid::Uuid, String, Option<String>)> = sqlx::query_as(
        "SELECT id, title, path FROM menus WHERE title ILIKE $1 OR path ILIKE $1 LIMIT $2",
    )
    .bind(&pattern)
    .bind(limit)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    for (id, title, path) in menus {
        results.push(SearchItem {
            r#type: "menu".into(),
            id: id.to_string(),
            title,
            subtitle: path.clone(),
            path: path.unwrap_or_else(|| "/dashboard/menus".into()),
        });
    }

    // Search system configs
    let configs: Vec<(String, String)> = sqlx::query_as(
        "SELECT config_key, value FROM system_configs WHERE config_key ILIKE $1 OR value ILIKE $1 LIMIT $2",
    )
    .bind(&pattern)
    .bind(limit)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    for (key, value) in configs {
        results.push(SearchItem {
            r#type: "config".into(),
            id: key.clone(),
            title: key,
            subtitle: Some(value.chars().take(50).collect()),
            path: "/dashboard/configs".into(),
        });
    }

    // Sort: exact matches first, then by type
    results.sort_by(|a, b| {
        let a_exact = a.title.to_lowercase().contains(&query.q.to_lowercase());
        let b_exact = b.title.to_lowercase().contains(&query.q.to_lowercase());
        b_exact.cmp(&a_exact)
    });

    results.truncate(20);

    Ok(Json(SearchResult { results }))
}
