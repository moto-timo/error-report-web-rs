use axum::{
    routing::{get, post},
    Router,
};
use sea_orm::{Database, DatabaseConnection};
use std::sync::Arc;
use tower_http::{
    services::ServeDir,
    trace::TraceLayer,
    cors::CorsLayer,
};
use tracing_subscriber;

mod config;
mod handlers;
mod models;
mod services;
mod utils;

use config::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub config: Arc<Config>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::init();

    // Load configuration
    dotenvy::dotenv().ok();
    let config = Arc::new(Config::from_env()?);

    // Connect to database
    let db = Database::connect(&config.database_url).await?;

    let app_state = AppState {
        db,
        config: config.clone(),
    };

    // Build application router
    let app = Router::new()
        // API routes - maintaining Django compatibility
        .route("/ClientPost/JSON/", post(handlers::api::submit_error_report))
        .route("/api/errors", get(handlers::api::list_errors))
        .route("/api/errors/:id", get(handlers::api::get_error))
        .route("/api/stats", get(handlers::api::get_stats))

        // Web interface routes
        .route("/", get(handlers::web::index))
        .route("/Errors", get(handlers::web::error_list_page))
        .route("/Errors/Details/:id/", get(handlers::web::error_detail_page))
        .route("/Stats", get(handlers::web::stats_page))
        .route("/Stats/", get(handlers::web::stats_page))

        // Admin routes
        .route("/admin", get(handlers::admin::admin_dashboard))
        .route("/admin/", get(handlers::admin::admin_dashboard))

        // Static file serving
        .nest_service("/static", ServeDir::new(&config.static_dir))

        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let bind_addr = format!("{}:{}", config.bind_address, config.port);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;

    tracing::info!("🚀 Error Report Web Server starting on http://{}", bind_addr);
    tracing::info!("📊 Dashboard available at http://{}/", bind_addr);
    tracing::info!("📋 API documentation at http://{}/api/", bind_addr);

    axum::serve(listener, app).await?;
    Ok(())
}
