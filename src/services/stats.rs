use chrono::{DateTime, Duration, Utc};
use sea_orm::{
    Alias, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, Expr, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect,
};
use serde::Serialize;

use crate::models::{error_report, ErrorReport};

#[derive(Debug, Serialize)]
pub struct ErrorStats {
    pub total_errors: u64,
    pub errors_by_type: Vec<(String, u64)>,
    pub errors_by_machine: Vec<(String, u64)>,
    pub errors_by_distro: Vec<(String, u64)>,
    pub errors_by_package: Vec<(String, u64)>,
    pub recent_errors: Vec<error_report::Model>,
    pub daily_stats: Vec<DailyStats>,
    pub top_submitters: Vec<(String, u64)>,
    pub error_trends: ErrorTrends,
}

#[derive(Debug, Serialize)]
pub struct DailyStats {
    pub date: String,
    pub count: u64,
}

#[derive(Debug, Serialize)]
pub struct ErrorTrends {
    pub this_week: u64,
    pub last_week: u64,
    pub this_month: u64,
    pub last_month: u64,
    pub week_over_week_change: f64,
    pub month_over_month_change: f64,
}

pub struct StatsService {
    db: DatabaseConnection,
}

impl StatsService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn get_error_stats(&self) -> Result<ErrorStats, DbErr> {
        // Total errors
        let total_errors = ErrorReport::find().count(&self.db).await?;

        // Errors by type
        let errors_by_type = self
            .get_grouped_stats(error_report::Column::ErrorType)
            .await?;

        // Errors by machine
        let errors_by_machine = self
            .get_grouped_stats(error_report::Column::Machine)
            .await?;

        // Errors by distro
        let errors_by_distro = self.get_grouped_stats(error_report::Column::Distro).await?;

        // Errors by package (top failures)
        let errors_by_package = self
            .get_grouped_stats(error_report::Column::FailurePackage)
            .await?;

        // Recent errors
        let recent_errors = ErrorReport::find()
            .order_by_desc(error_report::Column::CreatedAt)
            .limit(10)
            .all(&self.db)
            .await?;

        // Daily stats for the last 30 days
        let daily_stats = self.get_daily_stats().await?;

        // Top submitters (by email)
        let top_submitters = self.get_top_submitters().await?;

        // Error trends
        let error_trends = self.get_error_trends().await?;

        Ok(ErrorStats {
            total_errors,
            errors_by_type,
            errors_by_machine,
            errors_by_distro,
            errors_by_package,
            recent_errors,
            daily_stats,
            top_submitters,
            error_trends,
        })
    }

    async fn get_grouped_stats(
        &self,
        column: error_report::Column,
    ) -> Result<Vec<(String, u64)>, DbErr> {
        let results = ErrorReport::find()
            .select_only()
            .column(column)
            .column_as(error_report::Column::Id.count(), "count")
            .group_by(column)
            .order_by_desc(Expr::col(Alias::new("count")))
            .limit(10)
            .into_tuple::<(String, i64)>()
            .all(&self.db)
            .await?;

        Ok(results
            .into_iter()
            .map(|(name, count)| (name, count as u64))
            .collect())
    }

    async fn get_daily_stats(&self) -> Result<Vec<DailyStats>, DbErr> {
        let thirty_days_ago = Utc::now() - Duration::days(30);

        // For now, return a simple implementation
        // In production, you might want to use raw SQL for better date handling
        let all_errors = ErrorReport::find()
            .filter(error_report::Column::CreatedAt.gte(thirty_days_ago))
            .all(&self.db)
            .await?;

        // Group by date manually
        let mut daily_counts = std::collections::HashMap::new();
        for error in all_errors {
            let date_str = error.created_at.format("%Y-%m-%d").to_string();
            *daily_counts.entry(date_str).or_insert(0) += 1;
        }

        let mut results: Vec<DailyStats> = daily_counts
            .into_iter()
            .map(|(date, count)| DailyStats { date, count })
            .collect();

        results.sort_by(|a, b| a.date.cmp(&b.date));
        Ok(results)
    }

    async fn get_top_submitters(&self) -> Result<Vec<(String, u64)>, DbErr> {
        let results = ErrorReport::find()
            .select_only()
            .column(error_report::Column::SubmitterEmail)
            .column_as(error_report::Column::Id.count(), "count")
            .filter(error_report::Column::SubmitterEmail.is_not_null())
            .group_by(error_report::Column::SubmitterEmail)
            .order_by_desc(Expr::col(Alias::new("count")))
            .limit(10)
            .into_tuple::<(Option<String>, i64)>()
            .all(&self.db)
            .await?;

        Ok(results
            .into_iter()
            .filter_map(|(email, count)| email.map(|e| (e, count as u64)))
            .collect())
    }

    async fn get_error_trends(&self) -> Result<ErrorTrends, DbErr> {
        let now = Utc::now();
        let week_ago = now - Duration::weeks(1);
        let two_weeks_ago = now - Duration::weeks(2);
        let month_ago = now - Duration::days(30);
        let two_months_ago = now - Duration::days(60);

        // This week
        let this_week = ErrorReport::find()
            .filter(error_report::Column::CreatedAt.gte(week_ago))
            .count(&self.db)
            .await?;

        // Last week
        let last_week = ErrorReport::find()
            .filter(error_report::Column::CreatedAt.gte(two_weeks_ago))
            .filter(error_report::Column::CreatedAt.lt(week_ago))
            .count(&self.db)
            .await?;

        // This month
        let this_month = ErrorReport::find()
            .filter(error_report::Column::CreatedAt.gte(month_ago))
            .count(&self.db)
            .await?;

        // Last month
        let last_month = ErrorReport::find()
            .filter(error_report::Column::CreatedAt.gte(two_months_ago))
            .filter(error_report::Column::CreatedAt.lt(month_ago))
            .count(&self.db)
            .await?;

        // Calculate percentage changes
        let week_over_week_change = if last_week > 0 {
            ((this_week as f64 - last_week as f64) / last_week as f64) * 100.0
        } else {
            0.0
        };

        let month_over_month_change = if last_month > 0 {
            ((this_month as f64 - last_month as f64) / last_month as f64) * 100.0
        } else {
            0.0
        };

        Ok(ErrorTrends {
            this_week,
            last_week,
            this_month,
            last_month,
            week_over_week_change,
            month_over_month_change,
        })
    }

    pub async fn get_machine_stats(&self) -> Result<Vec<(String, u64)>, DbErr> {
        self.get_grouped_stats(error_report::Column::Machine).await
    }

    pub async fn get_distro_stats(&self) -> Result<Vec<(String, u64)>, DbErr> {
        self.get_grouped_stats(error_report::Column::Distro).await
    }

    pub async fn get_error_type_stats(&self) -> Result<Vec<(String, u64)>, DbErr> {
        self.get_grouped_stats(error_report::Column::ErrorType)
            .await
    }

    pub async fn get_package_failure_stats(&self) -> Result<Vec<(String, u64)>, DbErr> {
        self.get_grouped_stats(error_report::Column::FailurePackage)
            .await
    }
}
