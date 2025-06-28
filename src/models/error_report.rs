use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "error_reports")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub created_at: DateTime<Utc>,
    pub machine: String,
    pub distro: String,
    pub distro_version: String,
    pub build_sys: String,
    pub nativelsbstring: String,
    pub target_sys: String,
    pub failure_task: String,
    pub failure_package: String,
    pub error_type: String,
    pub error_details: String,
    pub log_data: String,
    pub submitter_name: Option<String>,
    pub submitter_email: Option<String>,
    pub bugzilla_link: Option<String>,
    pub branch_commit: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::build_configuration::Entity")]
    BuildConfiguration,
}

impl Related<super::build_configuration::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::BuildConfiguration.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// DTO for API requests
#[derive(Debug, Deserialize, Serialize)]
pub struct ErrorSubmissionData {
    pub machine: String,
    pub distro: String,
    pub distro_version: String,
    pub build_sys: String,
    pub nativelsbstring: String,
    pub target_sys: String,
    pub failure_task: String,
    pub failure_package: String,
    pub error_type: String,
    pub error_details: String,
    pub log_data: String,
    pub submitter_name: Option<String>,
    pub submitter_email: Option<String>,
    pub branch_commit: String,
    pub build_configuration: Option<super::build_configuration::BuildConfigData>,
}

// Response DTOs
#[derive(Debug, Serialize)]
pub struct SubmissionResponse {
    pub id: i32,
    pub url: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorListResponse {
    pub errors: Vec<Model>,
    pub pagination: PaginationInfo,
}

#[derive(Debug, Serialize)]
pub struct PaginationInfo {
    pub page: u64,
    pub per_page: u64,
    pub total: u64,
    pub total_pages: u64,
}

// Query parameters for filtering
#[derive(Debug, Deserialize)]
pub struct ErrorQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub machine: Option<String>,
    pub distro: Option<String>,
    pub distro_version: Option<String>,
    pub error_type: Option<String>,
    pub failure_package: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub search: Option<String>,
}

impl Default for ErrorQuery {
    fn default() -> Self {
        Self {
            page: Some(1),
            per_page: Some(50),
            machine: None,
            distro: None,
            distro_version: None,
            error_type: None,
            failure_package: None,
            date_from: None,
            date_to: None,
            search: None,
        }
    }
}
