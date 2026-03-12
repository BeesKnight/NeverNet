use chrono::NaiveDate;
use serde::Serialize;
use uuid::Uuid;

use crate::events::models::{EventFilters, EventListItem};

#[derive(Debug, Clone, Serialize)]
pub struct CategoryReportRow {
    pub category_id: Uuid,
    pub category_name: String,
    pub category_color: String,
    pub event_count: usize,
    pub total_budget: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatusReportRow {
    pub status: String,
    pub event_count: usize,
    pub total_budget: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReportSummary {
    pub filters: EventFilters,
    pub period_start: Option<NaiveDate>,
    pub period_end: Option<NaiveDate>,
    pub total_events: usize,
    pub total_budget: f64,
    pub by_category: Vec<CategoryReportRow>,
    pub by_status: Vec<StatusReportRow>,
    pub events: Vec<EventListItem>,
}
