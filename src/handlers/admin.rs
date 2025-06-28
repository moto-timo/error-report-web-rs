use askama::Template;
use axum::{extract::State, http::StatusCode, response::Html};
use sea_orm::EntityTrait;
use tracing::error;

use crate::{models::ErrorReport, services::stats::StatsService, AppState};

#[derive(Template)]
#[template(path = "admin_dashboard.html")]
pub struct AdminDashboardTemplate {
    pub stats: crate::services::stats::ErrorStats,
    pub base_url: String,
}

/// Admin dashboard with comprehensive statistics
pub async fn admin_dashboard(
    State(app_state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let stats_service = StatsService::new(app_state.db.clone());

    let stats = stats_service.get_error_stats().await.map_err(|e| {
        error!("Failed to get admin stats: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let template = AdminDashboardTemplate {
        stats,
        base_url: app_state.config.base_url.clone(),
    };

    let html = template.render().map_err(|e| {
        error!("Admin template rendering failed: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Html(html))
}
