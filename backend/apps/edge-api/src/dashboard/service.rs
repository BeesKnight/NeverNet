use chrono::{DateTime, Utc};
use contracts::event_query::GetDashboardRequest;
use contracts::event_query::event_query_service_client::EventQueryServiceClient;
use tonic::transport::Channel;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    dashboard::models::{DashboardCards, DashboardResponse, RecentActivityItem},
    error::AppError,
    events::models::EventListItem,
    shared::grpc,
};

pub async fn get(state: &AppState, user_id: Uuid) -> Result<DashboardResponse, AppError> {
    let mut client = query_client(state).await?;
    let reply = client
        .get_dashboard(GetDashboardRequest {
            user_id: user_id.to_string(),
        })
        .await
        .map_err(map_status)?
        .into_inner();

    let cards = reply.cards.ok_or_else(|| {
        AppError::Internal("Query response is missing dashboard cards".to_string())
    })?;

    Ok(DashboardResponse {
        cards: DashboardCards {
            total_events: cards.total_events,
            upcoming_events: cards.upcoming_events,
            completed_events: cards.completed_events,
            cancelled_events: cards.cancelled_events,
            total_budget: cards.total_budget,
        },
        upcoming: reply
            .upcoming
            .into_iter()
            .map(map_event)
            .collect::<Result<_, _>>()?,
        recent_activity: reply
            .recent_activity
            .into_iter()
            .map(map_activity)
            .collect::<Result<_, _>>()?,
    })
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

fn map_activity(
    item: contracts::event_query::ActivityItem,
) -> Result<RecentActivityItem, AppError> {
    Ok(RecentActivityItem {
        id: item.id,
        entity_type: item.entity_type,
        entity_id: item.entity_id,
        action: item.action,
        title: item.title,
        occurred_at: parse_timestamp(&item.occurred_at, "activity occurred_at")?,
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

fn map_status(status: tonic::Status) -> AppError {
    match status.code() {
        tonic::Code::InvalidArgument => AppError::BadRequest(status.message().to_string()),
        tonic::Code::NotFound => AppError::NotFound(status.message().to_string()),
        _ => AppError::Internal(format!("Dashboard service error: {}", status.message())),
    }
}
