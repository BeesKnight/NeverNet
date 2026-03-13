use contracts::event_query::GetCalendarRequest;
use contracts::event_query::event_query_service_client::EventQueryServiceClient;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    calendar::models::{CalendarFilters, CalendarItem},
    error::AppError,
};

pub async fn list(
    state: &AppState,
    user_id: Uuid,
    filters: CalendarFilters,
) -> Result<Vec<CalendarItem>, AppError> {
    let mut client = query_client(state).await?;
    let reply = client
        .get_calendar(GetCalendarRequest {
            user_id: user_id.to_string(),
            start_date: filters.start_date,
            end_date: filters.end_date,
        })
        .await
        .map_err(map_status)?
        .into_inner();

    Ok(reply
        .items
        .into_iter()
        .map(|item| CalendarItem {
            event_id: item.event_id,
            title: item.title,
            date: item.date,
            starts_at: item.starts_at,
            ends_at: item.ends_at,
            status: item.status,
            category_color: item.category_color,
        })
        .collect())
}

async fn query_client(
    state: &AppState,
) -> Result<EventQueryServiceClient<tonic::transport::Channel>, AppError> {
    EventQueryServiceClient::connect(state.config.event_query_service_url.clone())
        .await
        .map_err(|error| AppError::Internal(format!("Event query service is unavailable: {error}")))
}

fn map_status(status: tonic::Status) -> AppError {
    match status.code() {
        tonic::Code::InvalidArgument => AppError::BadRequest(status.message().to_string()),
        tonic::Code::NotFound => AppError::NotFound(status.message().to_string()),
        _ => AppError::Internal(format!("Calendar service error: {}", status.message())),
    }
}
