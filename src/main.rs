use error_report_web_rs::{create_app, init_logging, AppState, Config};
use sea_orm::Database;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    init_logging();

    // Load configuration
    dotenvy::dotenv().ok();
    let config = Arc::new(Config::from_env()?);

    // Connect to database
    let db = Database::connect(&config.database_url).await?;

    let app_state = AppState {
        db,
        config: config.clone(),
    };

    // Build application
    let app = create_app(app_state);

    let bind_addr = format!("{}:{}", config.bind_address, config.port);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;

    tracing::info!(
        "🚀 Error Report Web Server starting on http://{}",
        bind_addr
    );
    tracing::info!("📊 Dashboard available at http://{}/", bind_addr);
    tracing::info!("📋 API documentation at http://{}/api/", bind_addr);

    axum::serve(listener, app).await?;
    Ok(())
}
