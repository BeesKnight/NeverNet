use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::events::models::EventFilters;

#[derive(Debug, Clone, Serialize)]
pub struct ExportJob {
    pub id: Uuid,
    pub user_id: Uuid,
    pub report_type: String,
    pub format: String,
    pub status: String,
    pub filters: serde_json::Value,
    pub object_key: Option<String>,
    pub content_type: Option<String>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
}

fn default_report_type() -> String {
    "summary".to_string()
}

#[derive(Debug, Deserialize)]
pub struct CreateExportRequest {
    #[serde(default = "default_report_type")]
    pub report_type: String,
    pub format: String,
    #[serde(default)]
    pub filters: EventFilters,
}
