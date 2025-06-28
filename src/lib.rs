use axum::{
    routing::{get, post},
    Router,
};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::sync::Arc;
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};

pub mod config;
pub mod handlers;
pub mod models;
pub mod services;
pub mod utils;

pub use config::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub config: Arc<Config>,
}

pub async fn create_app(config: Config) -> Router {
    let db = Database::connect(&config.database_url)
        .await
        .expect("Failed to connect to database");

    let app_state = AppState {
        db,
        config: Arc::new(config),
    };

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
        .route(
            "/Errors/Details/:id/",
            get(handlers::web::error_detail_page),
        )
        .route("/Stats", get(handlers::web::stats_page))
        .route("/Stats/", get(handlers::web::stats_page))
        // Admin routes
        .route("/admin", get(handlers::admin::admin_dashboard))
        .route("/admin/", get(handlers::admin::admin_dashboard))
        // Static file serving
        .nest_service("/static", ServeDir::new(&app_state.config.static_dir))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(app_state)
}

#[cfg(test)]
pub async fn create_test_app() -> Router {
    use std::env;

    // Set up test database URL - use in-memory SQLite for tests
    let test_db_url =
        env::var("TEST_DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".to_string());

    let config = Config {
        database_url: test_db_url,
        bind_address: "127.0.0.1".to_string(),
        port: 8000,
        base_url: "http://localhost:8000".to_string(),
        static_dir: "./static".to_string(),
        template_dir: "./templates".to_string(),
        max_upload_size: 10485760,
        bugzilla_url: "https://bugzilla.yoctoproject.org".to_string(),
        email: config::EmailConfig {
            host: "localhost".to_string(),
            port: 587,
            username: None,
            password: None,
            from_address: "test@example.com".to_string(),
        },
    };

    // Create database connection with special options for testing
    let mut opt = ConnectOptions::new(&config.database_url);
    opt.max_connections(100)
        .min_connections(5)
        .sqlx_logging(false); // Disable logging in tests

    let db = Database::connect(opt)
        .await
        .expect("Failed to connect to test database");

    // Run migrations for test database
    // In a real implementation, you'd want to run the schema creation here

    let app_state = AppState {
        db,
        config: Arc::new(config),
    };

    Router::new()
        .route(
            "/ClientPost/JSON/",
            post(handlers::api::submit_error_report),
        )
        .route("/api/errors", get(handlers::api::list_errors))
        .route("/api/errors/:id", get(handlers::api::get_error))
        .route("/api/stats", get(handlers::api::get_stats))
        .route("/", get(handlers::web::index))
        .route("/Errors", get(handlers::web::error_list_page))
        .route(
            "/Errors/Details/:id/",
            get(handlers::web::error_detail_page),
        )
        .route("/Stats", get(handlers::web::stats_page))
        .route("/admin", get(handlers::admin::admin_dashboard))
        .with_state(app_state)
}
