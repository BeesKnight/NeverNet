use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
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

#[derive(Debug, Serialize)]
pub struct ExportEventPayload {
    pub export_id: Uuid,
    pub user_id: Uuid,
    pub report_type: String,
    pub format: String,
    pub status: String,
    pub filters: serde_json::Value,
    pub object_key: Option<String>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
}
