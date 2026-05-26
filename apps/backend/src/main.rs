use std::net::SocketAddr;

use cradle_backend::app;
use cradle_backend::config::Settings;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "cradle_backend=debug,tower_http=debug".into()),
        )
        .init();

    // Load config
    let settings = Settings::new()?;

    // Build app
    let app = app(&settings).await?;

    // Start server with ConnectInfo for rate limiting IP extraction
    let addr = format!("{}:{}", settings.server.host, settings.server.port);
    tracing::info!("Server running on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
