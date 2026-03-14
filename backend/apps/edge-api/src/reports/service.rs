use chrono::{DateTime, NaiveDate, Utc};
use contracts::event_query::GetReportSummaryRequest;
use contracts::event_query::event_query_service_client::EventQueryServiceClient;
use tonic::transport::Channel;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    error::AppError,
    events::models::{EventFilters, EventListItem},
    reports::models::{CategoryReportRow, ReportSummary, StatusReportRow},
    shared::grpc,
};

pub async fn generate_summary(
    state: &AppState,
    user_id: Uuid,
    filters: EventFilters,
) -> Result<ReportSummary, AppError> {
    let mut client = query_client(state).await?;
    let reply = client
        .get_report_summary(GetReportSummaryRequest {
            user_id: user_id.to_string(),
            category_id: filters
                .category_id
                .map(|value| value.to_string())
                .unwrap_or_default(),
            status: filters.status.clone().unwrap_or_default(),
            start_date: filters
                .start_date
                .map(|value| value.to_string())
                .unwrap_or_default(),
            end_date: filters
                .end_date
                .map(|value| value.to_string())
                .unwrap_or_default(),
            sort_by: filters.sort_by.clone().unwrap_or_default(),
            sort_dir: filters.sort_dir.clone().unwrap_or_default(),
        })
        .await
        .map_err(map_status)?
        .into_inner();

    Ok(ReportSummary {
        filters,
        period_start: optional_date(&reply.period_start, "period_start")?,
        period_end: optional_date(&reply.period_end, "period_end")?,
        total_events: reply.total_events as usize,
        total_budget: reply.total_budget,
        by_category: reply
            .by_category
            .into_iter()
            .map(|row| {
                Ok(CategoryReportRow {
                    category_id: parse_uuid(&row.category_id, "report category id")?,
                    category_name: row.category_name,
                    category_color: row.category_color,
                    event_count: row.event_count as usize,
                    total_budget: row.total_budget,
                })
            })
            .collect::<Result<_, AppError>>()?,
        by_status: reply
            .by_status
            .into_iter()
            .map(|row| StatusReportRow {
                status: row.status,
                event_count: row.event_count as usize,
                total_budget: row.total_budget,
            })
            .collect(),
        events: reply
            .events
            .into_iter()
            .map(map_event)
            .collect::<Result<_, _>>()?,
    })
}

pub async fn generate_by_category(
    state: &AppState,
    user_id: Uuid,
    filters: EventFilters,
) -> Result<Vec<CategoryReportRow>, AppError> {
    Ok(generate_summary(state, user_id, filters).await?.by_category)
}

async fn query_client(
    state: &AppState,
) -> Result<
    EventQueryServiceClient<
        tonic::service::interceptor::InterceptedService<Channel, grpc::RequestIdInterceptor>,
    >,
    AppError,
> {
    let channel =
        grpc::connect_channel(&state.config.event_query_service_url, "Event query service").await?;

    Ok(EventQueryServiceClient::with_interceptor(
        channel,
        grpc::RequestIdInterceptor,
    ))
}

fn map_event(event: contracts::event_query::EventItem) -> Result<EventListItem, AppError> {
    Ok(EventListItem {
        id: parse_uuid(&event.id, "event id")?,
        user_id: parse_uuid(&event.user_id, "event user id")?,
        category_id: parse_uuid(&event.category_id, "event category id")?,
        category_name: event.category_name,
        category_color: event.category_color,
        title: event.title,
        description: event.description,
        location: event.location,
        starts_at: parse_timestamp(&event.starts_at, "event starts_at")?,
        ends_at: parse_timestamp(&event.ends_at, "event ends_at")?,
        budget: event.budget,
        status: event.status,
        created_at: parse_timestamp(&event.created_at, "event created_at")?,
        updated_at: parse_timestamp(&event.updated_at, "event updated_at")?,
    })
}

fn parse_uuid(value: &str, field: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(value)
        .map_err(|_| AppError::Internal(format!("Internal service returned an invalid {field}")))
}

fn parse_timestamp(value: &str, field: &str) -> Result<DateTime<Utc>, AppError> {
    DateTime::parse_from_rfc3339(value)
        .map(|timestamp| timestamp.with_timezone(&Utc))
        .map_err(|_| AppError::Internal(format!("Internal service returned an invalid {field}")))
}

fn optional_date(value: &str, field: &str) -> Result<Option<NaiveDate>, AppError> {
    if value.is_empty() {
        Ok(None)
    } else {
        NaiveDate::parse_from_str(value, "%Y-%m-%d")
            .map(Some)
            .map_err(|_| {
                AppError::Internal(format!("Internal service returned an invalid {field}"))
            })
    }
}

fn map_status(status: tonic::Status) -> AppError {
    match status.code() {
        tonic::Code::InvalidArgument => AppError::BadRequest(status.message().to_string()),
        tonic::Code::NotFound => AppError::NotFound(status.message().to_string()),
        _ => AppError::Internal(format!("Report query service error: {}", status.message())),
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    #[test]
    fn maps_report_events_and_optional_dates() {
        let event = map_event(contracts::event_query::EventItem {
            id: Uuid::new_v4().to_string(),
            user_id: Uuid::new_v4().to_string(),
            category_id: Uuid::new_v4().to_string(),
            category_name: "Conference".to_string(),
            category_color: "#0f766e".to_string(),
            title: "Defense rehearsal".to_string(),
            description: "Dry run".to_string(),
            location: "Room 301".to_string(),
            starts_at: Utc
                .with_ymd_and_hms(2026, 3, 15, 10, 0, 0)
                .unwrap()
                .to_rfc3339(),
            ends_at: Utc
                .with_ymd_and_hms(2026, 3, 15, 12, 0, 0)
                .unwrap()
                .to_rfc3339(),
            budget: 850.0,
            status: "planned".to_string(),
            created_at: Utc
                .with_ymd_and_hms(2026, 3, 13, 10, 0, 0)
                .unwrap()
                .to_rfc3339(),
            updated_at: Utc
                .with_ymd_and_hms(2026, 3, 13, 10, 5, 0)
                .unwrap()
                .to_rfc3339(),
        })
        .expect("event should map");

        assert_eq!(event.title, "Defense rehearsal");
        assert!(
            optional_date("", "period_start")
                .expect("empty date")
                .is_none()
        );
        assert!(optional_date("bad", "period_start").is_err());
    }

    #[test]
    fn maps_report_status_codes() {
        assert!(matches!(
            map_status(tonic::Status::invalid_argument("bad")),
            AppError::BadRequest(_)
        ));
        assert!(matches!(
            map_status(tonic::Status::not_found("missing")),
            AppError::NotFound(_)
        ));
        assert!(matches!(
            map_status(tonic::Status::internal("oops")),
            AppError::Internal(_)
        ));
    }
}
