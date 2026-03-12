use axum::{
    Json,
    extract::{Path, Query, State},
};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    error::AppError,
    events::{
        models::{CreateEventRequest, Event, EventFilters, EventListItem, UpdateEventRequest},
        service,
    },
    shared::{api::ApiResponse, auth::CurrentUser},
};

pub async fn list(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Query(filters): Query<EventFilters>,
) -> Result<Json<ApiResponse<Vec<EventListItem>>>, AppError> {
    let events = service::list(&state, current_user.user_id, filters).await?;
    Ok(Json(ApiResponse::new(events)))
}

pub async fn get(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(event_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Event>>, AppError> {
    let event = service::get(&state, current_user.user_id, event_id).await?;
    Ok(Json(ApiResponse::new(event)))
}

pub async fn create(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(payload): Json<CreateEventRequest>,
) -> Result<Json<ApiResponse<Event>>, AppError> {
    let event = service::create(&state, current_user.user_id, payload).await?;
    Ok(Json(ApiResponse::new(event)))
}

pub async fn update(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(event_id): Path<Uuid>,
    Json(payload): Json<UpdateEventRequest>,
) -> Result<Json<ApiResponse<Event>>, AppError> {
    let event = service::update(&state, current_user.user_id, event_id, payload).await?;
    Ok(Json(ApiResponse::new(event)))
}

pub async fn delete(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(event_id): Path<Uuid>,
) -> Result<Json<ApiResponse<&'static str>>, AppError> {
    service::delete(&state, current_user.user_id, event_id).await?;
    Ok(Json(ApiResponse::new("deleted")))
}
