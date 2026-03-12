use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::events::models::EventFilters;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ExportJob {
    pub id: Uuid,
    pub user_id: Uuid,
    pub format: String,
    pub status: String,
    pub filters: serde_json::Value,
    pub file_path: Option<String>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateExportRequest {
    pub format: String,
    #[serde(default)]
    pub filters: EventFilters,
}
