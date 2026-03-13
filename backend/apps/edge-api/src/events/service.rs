use chrono::{DateTime, Utc};
use contracts::event_command::event_command_service_client::EventCommandServiceClient;
use contracts::event_command::{
    CreateEventRequest as CommandCreateEventRequest,
    UpdateEventRequest as CommandUpdateEventRequest,
};
use contracts::event_query::event_query_service_client::EventQueryServiceClient;
use contracts::event_query::{
    GetEventRequest as QueryGetEventRequest, ListEventsRequest as QueryListEventsRequest,
};
use tonic::transport::Channel;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    error::AppError,
    events::models::{CreateEventRequest, Event, EventFilters, EventListItem, UpdateEventRequest},
    shared::grpc,
};

pub async fn list(
    state: &AppState,
    user_id: Uuid,
    filters: EventFilters,
) -> Result<Vec<EventListItem>, AppError> {
    let mut client = query_client(state).await?;
    let reply = client
        .list_events(QueryListEventsRequest {
            user_id: user_id.to_string(),
            search: filters.search.unwrap_or_default(),
            status: filters.status.unwrap_or_default(),
            category_id: filters
                .category_id
                .map(|value| value.to_string())
                .unwrap_or_default(),
            start_date: filters
                .start_date
                .map(|value| value.to_string())
                .unwrap_or_default(),
            end_date: filters
                .end_date
                .map(|value| value.to_string())
                .unwrap_or_default(),
            sort_by: filters.sort_by.unwrap_or_default(),
            sort_dir: filters.sort_dir.unwrap_or_default(),
        })
        .await
        .map_err(map_status)?
        .into_inner();

    reply.items.into_iter().map(map_query_event).collect()
}

pub async fn get(state: &AppState, user_id: Uuid, event_id: Uuid) -> Result<Event, AppError> {
    let mut client = query_client(state).await?;
    let reply = client
        .get_event(QueryGetEventRequest {
            user_id: user_id.to_string(),
            event_id: event_id.to_string(),
        })
        .await
        .map_err(map_status)?
        .into_inner();

    let event = reply
        .event
        .ok_or_else(|| AppError::Internal("Query response is missing event".to_string()))?;

    Ok(Event {
        id: parse_uuid(&event.id, "event id")?,
        user_id: parse_uuid(&event.user_id, "event user id")?,
        category_id: parse_uuid(&event.category_id, "event category id")?,
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

pub async fn create(
    state: &AppState,
    user_id: Uuid,
    payload: CreateEventRequest,
) -> Result<Event, AppError> {
    let mut client = command_client(state).await?;
    let reply = client
        .create_event(CommandCreateEventRequest {
            user_id: user_id.to_string(),
            category_id: payload.category_id.to_string(),
            title: payload.title,
            description: payload.description,
            location: payload.location,
            starts_at: payload.starts_at.to_rfc3339(),
            ends_at: payload.ends_at.to_rfc3339(),
            budget: payload.budget,
            status: payload.status.as_str().to_string(),
        })
        .await
        .map_err(map_status)?
        .into_inner();

    map_command_event(
        reply
            .event
            .ok_or_else(|| AppError::Internal("Command response is missing event".to_string()))?,
    )
}

pub async fn update(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    payload: UpdateEventRequest,
) -> Result<Event, AppError> {
    let mut client = command_client(state).await?;
    let reply = client
        .update_event(CommandUpdateEventRequest {
            user_id: user_id.to_string(),
            event_id: event_id.to_string(),
            category_id: payload.category_id.to_string(),
            title: payload.title,
            description: payload.description,
            location: payload.location,
            starts_at: payload.starts_at.to_rfc3339(),
            ends_at: payload.ends_at.to_rfc3339(),
            budget: payload.budget,
            status: payload.status.as_str().to_string(),
        })
        .await
        .map_err(map_status)?
        .into_inner();

    map_command_event(
        reply
            .event
            .ok_or_else(|| AppError::Internal("Command response is missing event".to_string()))?,
    )
}

pub async fn delete(state: &AppState, user_id: Uuid, event_id: Uuid) -> Result<(), AppError> {
    let mut client = command_client(state).await?;
    client
        .delete_event(contracts::event_command::DeleteEventRequest {
            user_id: user_id.to_string(),
            event_id: event_id.to_string(),
        })
        .await
        .map_err(map_status)?;

    Ok(())
}

async fn command_client(
    state: &AppState,
) -> Result<
    EventCommandServiceClient<
        tonic::service::interceptor::InterceptedService<Channel, grpc::RequestIdInterceptor>,
    >,
    AppError,
> {
    let channel = grpc::connect_channel(
        &state.config.event_command_service_url,
        "Event command service",
    )
    .await?;

    Ok(EventCommandServiceClient::with_interceptor(
        channel,
        grpc::RequestIdInterceptor,
    ))
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

fn map_query_event(event: contracts::event_query::EventItem) -> Result<EventListItem, AppError> {
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

fn map_command_event(event: contracts::event_command::EventRecord) -> Result<Event, AppError> {
    Ok(Event {
        id: parse_uuid(&event.id, "event id")?,
        user_id: parse_uuid(&event.user_id, "event user id")?,
        category_id: parse_uuid(&event.category_id, "event category id")?,
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

fn map_status(status: tonic::Status) -> AppError {
    match status.code() {
        tonic::Code::InvalidArgument => AppError::BadRequest(status.message().to_string()),
        tonic::Code::NotFound => AppError::NotFound(status.message().to_string()),
        tonic::Code::AlreadyExists => AppError::Conflict(status.message().to_string()),
        _ => AppError::Internal(format!("Event service error: {}", status.message())),
    }
}
