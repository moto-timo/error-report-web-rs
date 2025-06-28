use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "build_configurations")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub error_report_id: i32,
    pub bb_version: String,
    pub tune_features: Option<String>,
    pub target_fpu: Option<String>,
    pub meta_layers: String, // JSON string
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::error_report::Entity",
        from = "Column::ErrorReportId",
        to = "super::error_report::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    ErrorReport,
}

impl Related<super::error_report::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ErrorReport.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// DTO for build configuration data in API requests
#[derive(Debug, Deserialize, Serialize)]
pub struct BuildConfigData {
    pub bb_version: String,
    pub tune_features: Option<String>,
    pub target_fpu: Option<String>,
    pub meta_layers: Option<Vec<LayerInfo>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LayerInfo {
    pub name: String,
    pub path: String,
    pub commit: Option<String>,
    pub branch: Option<String>,
}
