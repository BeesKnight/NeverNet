use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventStatus {
    Planned,
    InProgress,
    Completed,
    Cancelled,
}

impl EventStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventStatus::Planned => "planned",
            EventStatus::InProgress => "in_progress",
            EventStatus::Completed => "completed",
            EventStatus::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Event {
    pub id: Uuid,
    pub user_id: Uuid,
    pub category_id: Uuid,
    pub title: String,
    pub description: String,
    pub location: String,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub budget: f64,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EventListItem {
    pub id: Uuid,
    pub user_id: Uuid,
    pub category_id: Uuid,
    pub category_name: String,
    pub category_color: String,
    pub title: String,
    pub description: String,
    pub location: String,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub budget: f64,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEventRequest {
    pub category_id: Uuid,
    pub title: String,
    pub description: String,
    pub location: String,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub budget: f64,
    pub status: EventStatus,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEventRequest {
    pub category_id: Uuid,
    pub title: String,
    pub description: String,
    pub location: String,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub budget: f64,
    pub status: EventStatus,
}

#[derive(Debug, Deserialize, Clone, Default, Serialize)]
pub struct EventFilters {
    pub search: Option<String>,
    pub status: Option<String>,
    pub category_id: Option<Uuid>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub sort_by: Option<String>,
    pub sort_dir: Option<String>,
}
