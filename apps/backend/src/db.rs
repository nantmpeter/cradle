use sqlx::postgres::{PgPool, PgPoolOptions};
use crate::config::DatabaseSettings;

pub async fn init_pool(settings: &DatabaseSettings) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(settings.max_connections)
        .connect(&settings.url)
        .await
}
