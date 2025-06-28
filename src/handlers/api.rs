use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set,
};
use tracing::{error, info};

use crate::{
    models::{
        error_report::{
            self, ErrorListResponse, ErrorQuery, ErrorSubmissionData, PaginationInfo,
            SubmissionResponse,
        },
        ErrorReport,
    },
    services::stats::ErrorStats,
    utils::validation::validate_error_submission,
    AppState,
};

/// Submit a new error report - maintains compatibility with Django endpoint
pub async fn submit_error_report(
    State(app_state): State<AppState>,
    Json(payload): Json<ErrorSubmissionData>,
) -> Result<Json<SubmissionResponse>, StatusCode> {
    info!(
        "Received error report submission for machine: {}",
        payload.machine
    );

    // Validate submission data
    if let Err(validation_error) = validate_error_submission(&payload) {
        error!("Validation failed: {:?}", validation_error);
        return Err(StatusCode::BAD_REQUEST);
    }

    // Create error report
    let error_report = error_report::ActiveModel {
        machine: Set(payload.machine.clone()),
        distro: Set(payload.distro.clone()),
        distro_version: Set(payload.distro_version.clone()),
        build_sys: Set(payload.build_sys.clone()),
        nativelsbstring: Set(payload.nativelsbstring.clone()),
        target_sys: Set(payload.target_sys.clone()),
        failure_task: Set(payload.failure_task.clone()),
        failure_package: Set(payload.failure_package.clone()),
        error_type: Set(payload.error_type.clone()),
        error_details: Set(payload.error_details.clone()),
        log_data: Set(payload.log_data.clone()),
        submitter_name: Set(payload.submitter_name.clone()),
        submitter_email: Set(payload.submitter_email.clone()),
        branch_commit: Set(payload.branch_commit.clone()),
        created_at: Set(Utc::now()),
        bugzilla_link: Set(None),
        ..Default::default()
    };

    let saved_report = error_report.insert(&app_state.db).await.map_err(|e| {
        error!("Failed to save error report: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Save build configuration if provided
    if let Some(build_config) = payload.build_configuration {
        use crate::models::{build_configuration, BuildConfiguration};

        let meta_layers_json =
            serde_json::to_string(&build_config.meta_layers).unwrap_or_else(|_| "[]".to_string());

        let build_config_model = build_configuration::ActiveModel {
            error_report_id: Set(saved_report.id),
            bb_version: Set(build_config.bb_version),
            tune_features: Set(build_config.tune_features),
            target_fpu: Set(build_config.target_fpu),
            meta_layers: Set(meta_layers_json),
            ..Default::default()
        };

        if let Err(e) = build_config_model.insert(&app_state.db).await {
            error!("Failed to save build configuration: {:?}", e);
            // Continue anyway as this is not critical
        }
    }

    info!(
        "Successfully saved error report with ID: {}",
        saved_report.id
    );

    let response = SubmissionResponse {
        id: saved_report.id,
        url: format!(
            "{}/Errors/Details/{}/",
            app_state.config.base_url, saved_report.id
        ),
        status: "success".to_string(),
    };

    Ok(Json(response))
}

/// List errors with filtering and pagination
pub async fn list_errors(
    State(app_state): State<AppState>,
    Query(params): Query<ErrorQuery>,
) -> Result<Json<ErrorListResponse>, StatusCode> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(50).min(100); // Cap at 100 per page

    let mut query = ErrorReport::find();

    // Apply filters
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

    // Search functionality
    if let Some(search_term) = &params.search {
        query = query.filter(
            error_report::Column::ErrorDetails
                .contains(search_term)
                .or(error_report::Column::FailurePackage.contains(search_term))
                .or(error_report::Column::FailureTask.contains(search_term)),
        );
    }

    // Date filtering
    if let Some(date_from) = &params.date_from {
        if let Ok(parsed_date) = chrono::DateTime::parse_from_rfc3339(date_from) {
            query =
                query.filter(error_report::Column::CreatedAt.gte(parsed_date.with_timezone(&Utc)));
        }
    }
    if let Some(date_to) = &params.date_to {
        if let Ok(parsed_date) = chrono::DateTime::parse_from_rfc3339(date_to) {
            query =
                query.filter(error_report::Column::CreatedAt.lte(parsed_date.with_timezone(&Utc)));
        }
    }

    let paginator = query
        .order_by_desc(error_report::Column::CreatedAt)
        .paginate(&app_state.db, per_page);

    let errors = paginator.fetch_page(page - 1).await.map_err(|e| {
        error!("Failed to fetch errors: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let total = paginator.num_items().await.map_err(|e| {
        error!("Failed to count errors: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let response = ErrorListResponse {
        errors,
        pagination: PaginationInfo {
            page,
            per_page,
            total,
            total_pages: (total + per_page - 1) / per_page,
        },
    };

    Ok(Json(response))
}

/// Get a specific error by ID
pub async fn get_error(
    State(app_state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<error_report::Model>, StatusCode> {
    let error = ErrorReport::find_by_id(id)
        .one(&app_state.db)
        .await
        .map_err(|e| {
            error!("Failed to fetch error {}: {:?}", id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(error))
}

/// Get error statistics
pub async fn get_stats(State(app_state): State<AppState>) -> Result<Json<ErrorStats>, StatusCode> {
    let stats_service = crate::services::stats::StatsService::new(app_state.db.clone());

    let stats = stats_service.get_error_stats().await.map_err(|e| {
        error!("Failed to get stats: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(stats))
}
