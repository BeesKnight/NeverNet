use std::collections::BTreeMap;

use uuid::Uuid;

use crate::{
    app_state::AppState,
    error::AppError,
    events::{models::EventFilters, repository as events_repository},
    reports::models::{CategoryReportRow, ReportSummary, StatusReportRow},
};

pub async fn generate_summary(
    state: &AppState,
    user_id: Uuid,
    filters: EventFilters,
) -> Result<ReportSummary, AppError> {
    let events = events_repository::list(&state.pool, user_id, &filters).await?;

    let total_events = events.len();
    let total_budget = events.iter().map(|event| event.budget).sum::<f64>();

    let mut category_map: BTreeMap<Uuid, CategoryReportRow> = BTreeMap::new();
    let mut status_map: BTreeMap<String, StatusReportRow> = BTreeMap::new();

    for event in &events {
        category_map
            .entry(event.category_id)
            .and_modify(|row| {
                row.event_count += 1;
                row.total_budget += event.budget;
            })
            .or_insert(CategoryReportRow {
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
            .or_insert(StatusReportRow {
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

    Ok(ReportSummary {
        period_start: filters.start_date,
        period_end: filters.end_date,
        filters,
        total_events,
        total_budget,
        by_category,
        by_status,
        events,
    })
}

pub async fn generate_by_category(
    state: &AppState,
    user_id: Uuid,
    filters: EventFilters,
) -> Result<Vec<CategoryReportRow>, AppError> {
    Ok(generate_summary(state, user_id, filters).await?.by_category)
}
