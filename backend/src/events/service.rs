use uuid::Uuid;

use crate::{
    app_state::AppState,
    categories::repository as categories_repository,
    error::AppError,
    events::{
        models::{CreateEventRequest, Event, EventFilters, EventListItem, UpdateEventRequest},
        repository, validation,
    },
};

pub async fn list(
    state: &AppState,
    user_id: Uuid,
    filters: EventFilters,
) -> Result<Vec<EventListItem>, AppError> {
    Ok(repository::list(&state.pool, user_id, &filters).await?)
}

pub async fn get(state: &AppState, user_id: Uuid, event_id: Uuid) -> Result<Event, AppError> {
    repository::find_by_id(&state.pool, user_id, event_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Event not found".to_string()))
}

pub async fn create(
    state: &AppState,
    user_id: Uuid,
    payload: CreateEventRequest,
) -> Result<Event, AppError> {
    validation::validate_event(
        &payload.title,
        &payload.location,
        payload.starts_at,
        payload.ends_at,
        payload.budget,
    )?;
    ensure_category_belongs_to_user(state, user_id, payload.category_id).await?;
    Ok(repository::create(&state.pool, user_id, &payload).await?)
}

pub async fn update(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
    payload: UpdateEventRequest,
) -> Result<Event, AppError> {
    validation::validate_event(
        &payload.title,
        &payload.location,
        payload.starts_at,
        payload.ends_at,
        payload.budget,
    )?;
    ensure_category_belongs_to_user(state, user_id, payload.category_id).await?;
    repository::update(&state.pool, user_id, event_id, &payload)
        .await?
        .ok_or_else(|| AppError::NotFound("Event not found".to_string()))
}

pub async fn delete(state: &AppState, user_id: Uuid, event_id: Uuid) -> Result<(), AppError> {
    let rows = repository::delete(&state.pool, user_id, event_id).await?;
    if rows == 0 {
        return Err(AppError::NotFound("Event not found".to_string()));
    }

    Ok(())
}

async fn ensure_category_belongs_to_user(
    state: &AppState,
    user_id: Uuid,
    category_id: Uuid,
) -> Result<(), AppError> {
    categories_repository::find_by_id(&state.pool, user_id, category_id)
        .await?
        .ok_or_else(|| {
            AppError::BadRequest("Category does not belong to the current user".to_string())
        })?;

    Ok(())
}
