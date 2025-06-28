//! Yocto Project Error Reporting Web Application - Rust Implementation
//!
//! A high-performance Rust rewrite of the Django-based Yocto Project Error Reporting
//! Web Application. This application serves as a central repository for build error
//! reports from Yocto Project builds, providing both an API for submitting errors
//! and a web interface for browsing them.

pub mod config;
pub mod handlers;
pub mod models;
pub mod services;
pub mod utils;

pub use config::Config;

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Router,
};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub config: Arc<Config>,
}

/// Create the main application router with all routes configured
pub fn create_app(app_state: AppState) -> Router {
    Router::new()
        // API routes - maintaining Django compatibility
        .route(
            "/ClientPost/JSON/",
            post(handlers::api::submit_error_report),
        )
        .route("/api/errors", get(handlers::api::list_errors))
        .route("/api/errors/:id", get(handlers::api::get_error))
        .route("/api/stats", get(handlers::api::get_stats))
        // Web interface routes
        .route("/", get(handlers::web::index))
        .route("/Errors", get(handlers::web::error_list_page))
        .route("/Errors/", get(handlers::web::error_list_page))
        .route(
            "/Errors/Details/:id/",
            get(handlers::web::error_detail_page),
        )
        .route("/Stats", get(handlers::web::stats_page))
        .route("/Stats/", get(handlers::web::stats_page))
        // Admin routes
        .route("/admin", get(handlers::admin::admin_dashboard))
        .route("/admin/", get(handlers::admin::admin_dashboard))
        // Health check endpoint
        .route("/health", get(health_check))
        // Static file serving
        .nest_service("/static", ServeDir::new(&app_state.config.static_dir))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(app_state)
}

/// Health check endpoint for monitoring
pub async fn health_check() -> Result<&'static str, StatusCode> {
    Ok("OK")
}

/// Initialize logging based on environment
pub fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        // This is a basic test to ensure the app can be created
        // In a real implementation, you'd set up a test database connection
        // For now, this is just a placeholder
    }
}
