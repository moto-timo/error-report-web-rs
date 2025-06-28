use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Html,
};
use askama::Template;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, PaginatorTrait};
use tracing::error;

use crate::{
    models::{
        error_report::{self, ErrorQuery},
        build_configuration::{self},
        ErrorReport, BuildConfiguration,
    },
    services::stats::StatsService,
    AppState,
};

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub recent_errors: Vec<error_report::Model>,
    pub total_errors: u64,
    pub base_url: String,
}

#[derive(Template)]
#[template(path = "error_list.html")]
pub struct ErrorListTemplate {
    pub errors: Vec<error_report::Model>,
    pub pagination: crate::models::error_report::PaginationInfo,
    pub filters: ErrorFilters,
    pub base_url: String,
}

#[derive(Template)]
#[template(path = "error_detail.html")]
pub struct ErrorDetailTemplate {
    pub error: error_report::Model,
    pub build_config: Option<build_configuration::Model>,
    pub similar_errors: Vec<error_report::Model>,
    pub base_url: String,
    pub bugzilla_url: String,
}

#[derive(Template)]
#[template(path = "stats.html")]
pub struct StatsTemplate {
    pub stats: crate::services::stats::ErrorStats,
    pub base_url: String,
}

#[derive(Debug)]
pub struct ErrorFilters {
    pub machine: Option<String>,
    pub distro: Option<String>,
    pub distro_version: Option<String>,
    pub error_type: Option<String>,
    pub failure_package: Option<String>,
    pub search: Option<String>,
}

impl ErrorFilters {
    pub fn from_query(query: &ErrorQuery) -> Self {
        Self {
            machine: query.machine.clone(),
            distro: query.distro.clone(),
            distro_version: query.distro_version.clone(),
            error_type: query.error_type.clone(),
            failure_package: query.failure_package.clone(),
            search: query.search.clone(),
        }
    }
}

/// Homepage with recent errors and basic stats
pub async fn index(State(app_state): State<AppState>) -> Result<Html<String>, StatusCode> {
    // Get recent errors
    let recent_errors = ErrorReport::find()
        .order_by_desc(error_report::Column::CreatedAt)
        .limit(10)
        .all(&app_state.db)
        .await
        .map_err(|e| {
            error!("Failed to fetch recent errors: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Get total count
    let total_errors = ErrorReport::find()
        .count(&app_state.db)
        .await
        .map_err(|e| {
            error!("Failed to count errors: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let template = IndexTemplate {
        recent_errors,
        total_errors,
        base_url: app_state.config.base_url.clone(),
    };

    let html = template.render().map_err(|e| {
        error!("Template rendering failed: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Html(html))
}

/// Error list page with filtering and pagination
pub async fn error_list_page(
    State(app_state): State<AppState>,
    Query(params): Query<ErrorQuery>,
) -> Result<Html<String>, StatusCode> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(25).min(100);

    let mut query = ErrorReport::find();

    // Apply filters (same logic as API)
    if let Some(machine) = &params.machine {
        query = query.filter(error_report::Column::Machine.eq(machine));
    }
    if let Some(distro) = &params.distro {
        query = query.filter(error_report::Column::Distro.eq(distro));
    }
    if let Some(distro_version) = &params.distro_version {
        query = query.filter(error_report::Column::DistroVersion.eq(distro_version));
    }
    if let Some(error_type) = &params.error_type {
        query = query.filter(error_report::Column::ErrorType.eq(error_type));
    }
    if let Some(failure_package) = &params.failure_package {
        query = query.filter(error_report::Column::FailurePackage.eq(failure_package));
    }
    if let Some(search_term) = &params.search {
        query = query.filter(
            error_report::Column::ErrorDetails
                .contains(search_term)
                .or(error_report::Column::FailurePackage.contains(search_term))
                .or(error_report::Column::FailureTask.contains(search_term)),
        );
    }

    let paginator = query
        .order_by_desc(error_report::Column::CreatedAt)
        .paginate(&app_state.db, per_page);

    let errors = paginator
        .fetch_page(page - 1)
        .await
        .map_err(|e| {
            error!("Failed to fetch errors: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let total = paginator.num_items().await.map_err(|e| {
        error!("Failed to count errors: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let template = ErrorListTemplate {
        errors,
        pagination: crate::models::error_report::PaginationInfo {
            page,
            per_page,
            total,
            total_pages: (total + per_page - 1) / per_page,
        },
        filters: ErrorFilters::from_query(&params),
        base_url: app_state.config.base_url.clone(),
    };

    let html = template.render().map_err(|e| {
        error!("Template rendering failed: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Html(html))
}

/// Error detail page
pub async fn error_detail_page(
    State(app_state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Html<String>, StatusCode> {
    // Get the error
    let error = ErrorReport::find_by_id(id)
        .one(&app_state.db)
        .await
        .map_err(|e| {
            error!("Failed to fetch error {}: {:?}", id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Get build configuration
    let build_config = BuildConfiguration::find()
        .filter(build_configuration::Column::ErrorReportId.eq(id))
        .one(&app_state.db)
        .await
        .map_err(|e| {
            error!("Failed to fetch build config for error {}: {:?}", id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Find similar errors (same error type and package)
    let similar_errors = ErrorReport::find()
        .filter(error_report::Column::ErrorType.eq(&error.error_type))
        .filter(error_report::Column::FailurePackage.eq(&error.failure_package))
        .filter(error_report::Column::Id.ne(id))
        .order_by_desc(error_report::Column::CreatedAt)
        .limit(5)
        .all(&app_state.db)
        .await
        .map_err(|e| {
            error!("Failed to fetch similar errors: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let template = ErrorDetailTemplate {
        error,
        build_config,
        similar_errors,
        base_url: app_state.config.base_url.clone(),
        bugzilla_url: app_state.config.bugzilla_url.clone(),
    };

    let html = template.render().map_err(|e| {
        error!("Template rendering failed: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Html(html))
}

/// Statistics page
pub async fn stats_page(State(app_state): State<AppState>) -> Result<Html<String>, StatusCode> {
    let stats_service = StatsService::new(app_state.db.clone());

    let stats = stats_service.get_error_stats().await.map_err(|e| {
        error!("Failed to get stats: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let template = StatsTemplate {
        stats,
        base_url: app_state.config.base_url.clone(),
    };

    let html = template.render().map_err(|e| {
        error!("Template rendering failed: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Html(html))
}
