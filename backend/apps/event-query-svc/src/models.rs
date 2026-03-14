use std::collections::BTreeMap;

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct CategoryRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub color: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct EventItemRow {
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

#[derive(Debug, Clone, FromRow)]
pub struct CalendarItemRow {
    pub event_id: Uuid,
    pub title: String,
    pub date: NaiveDate,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub status: String,
    pub category_color: String,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct DashboardProjectionRow {
    pub user_id: Uuid,
    pub total_events: i64,
    pub upcoming_events: i64,
    pub completed_events: i64,
    pub cancelled_events: i64,
    pub total_budget: f64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ActivityRow {
    pub id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub action: String,
    pub title: String,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default)]
pub struct EventFilters {
    pub search: Option<String>,
    pub status: Option<String>,
    pub category_id: Option<Uuid>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub sort_by: Option<String>,
    pub sort_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSnapshot {
    pub cards: DashboardProjectionRow,
    pub upcoming: Vec<EventItemRow>,
    pub recent_activity: Vec<ActivityRow>,
}

#[derive(Debug, Clone)]
pub struct ReportSummary {
    pub period_start: Option<NaiveDate>,
    pub period_end: Option<NaiveDate>,
    pub total_events: usize,
    pub total_budget: f64,
    pub by_category: Vec<ReportCategoryRow>,
    pub by_status: Vec<ReportStatusRow>,
    pub events: Vec<EventItemRow>,
}

#[derive(Debug, Clone)]
pub struct ReportCategoryRow {
    pub category_id: Uuid,
    pub category_name: String,
    pub category_color: String,
    pub event_count: i64,
    pub total_budget: f64,
}

#[derive(Debug, Clone)]
pub struct ReportStatusRow {
    pub status: String,
    pub event_count: i64,
    pub total_budget: f64,
}

impl ReportSummary {
    pub fn from_events(
        events: Vec<EventItemRow>,
        period_start: Option<NaiveDate>,
        period_end: Option<NaiveDate>,
    ) -> Self {
        let total_events = events.len();
        let total_budget = events.iter().map(|event| event.budget).sum::<f64>();

        let mut category_map: BTreeMap<Uuid, ReportCategoryRow> = BTreeMap::new();
        let mut status_map: BTreeMap<String, ReportStatusRow> = BTreeMap::new();

        for event in &events {
            category_map
                .entry(event.category_id)
                .and_modify(|row| {
                    row.event_count += 1;
                    row.total_budget += event.budget;
                })
                .or_insert(ReportCategoryRow {
                    category_id: event.category_id,
                    category_name: event.category_name.clone(),
                    category_color: event.category_color.clone(),
                    event_count: 1,
                    total_budget: event.budget,
                });

            status_map
                .entry(event.status.clone())
                .and_modify(|row| {
                    row.event_count += 1;
                    row.total_budget += event.budget;
                })
                .or_insert(ReportStatusRow {
                    status: event.status.clone(),
                    event_count: 1,
                    total_budget: event.budget,
                });
        }

        let mut by_category: Vec<_> = category_map.into_values().collect();
        by_category.sort_by(|left, right| {
            right
                .total_budget
                .total_cmp(&left.total_budget)
                .then_with(|| right.event_count.cmp(&left.event_count))
                .then_with(|| left.category_name.cmp(&right.category_name))
        });

        let mut by_status: Vec<_> = status_map.into_values().collect();
        by_status.sort_by(|left, right| {
            right
                .event_count
                .cmp(&left.event_count)
                .then_with(|| right.total_budget.total_cmp(&left.total_budget))
                .then_with(|| left.status.cmp(&right.status))
        });

        Self {
            period_start,
            period_end,
            total_events,
            total_budget,
            by_category,
            by_status,
            events,
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn event_row(
        category_id: Uuid,
        category_name: &str,
        status: &str,
        budget: f64,
    ) -> EventItemRow {
        EventItemRow {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id,
            category_name: category_name.to_string(),
            category_color: "#0f766e".to_string(),
            title: "Demo event".to_string(),
            description: "Demo".to_string(),
            location: "Room 301".to_string(),
            starts_at: Utc.with_ymd_and_hms(2026, 3, 14, 10, 0, 0).unwrap(),
            ends_at: Utc.with_ymd_and_hms(2026, 3, 14, 12, 0, 0).unwrap(),
            budget,
            status: status.to_string(),
            created_at: Utc.with_ymd_and_hms(2026, 3, 13, 10, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2026, 3, 13, 10, 0, 0).unwrap(),
        }
    }

    #[test]
    fn aggregates_report_summary_by_category_and_status() {
        let conference = Uuid::new_v4();
        let meetup = Uuid::new_v4();
        let summary = ReportSummary::from_events(
            vec![
                event_row(conference, "Conference", "planned", 1200.0),
                event_row(conference, "Conference", "completed", 500.0),
                event_row(meetup, "Meetup", "planned", 800.0),
            ],
            Some(NaiveDate::from_ymd_opt(2026, 3, 1).expect("valid start")),
            Some(NaiveDate::from_ymd_opt(2026, 3, 31).expect("valid end")),
        );

        assert_eq!(summary.total_events, 3);
        assert_eq!(summary.total_budget, 2500.0);
        assert_eq!(summary.by_category[0].category_name, "Conference");
        assert_eq!(summary.by_category[0].event_count, 2);
        assert_eq!(summary.by_status[0].status, "planned");
        assert_eq!(summary.by_status[0].event_count, 2);
        assert_eq!(summary.period_start, NaiveDate::from_ymd_opt(2026, 3, 1));
        assert_eq!(summary.period_end, NaiveDate::from_ymd_opt(2026, 3, 31));
    }
}
