use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CalendarFilters {
    pub start_date: String,
    pub end_date: String,
}

#[derive(Debug, Serialize)]
pub struct CalendarItem {
    pub event_id: String,
    pub title: String,
    pub date: String,
    pub starts_at: String,
    pub ends_at: String,
    pub status: String,
    pub category_color: String,
}
