use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::events::models::EventListItem;

#[derive(Debug, Serialize)]
pub struct DashboardCards {
    pub total_events: i64,
    pub upcoming_events: i64,
    pub completed_events: i64,
    pub cancelled_events: i64,
    pub total_budget: f64,
}

#[derive(Debug, Serialize)]
pub struct RecentActivityItem {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub action: String,
    pub title: String,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct DashboardResponse {
    pub cards: DashboardCards,
    pub upcoming: Vec<EventListItem>,
    pub recent_activity: Vec<RecentActivityItem>,
}
