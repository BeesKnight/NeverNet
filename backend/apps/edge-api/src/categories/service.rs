use chrono::{DateTime, Utc};
use contracts::event_command::event_command_service_client::EventCommandServiceClient;
use contracts::event_command::{
    CreateCategoryRequest as CommandCreateCategoryRequest,
    UpdateCategoryRequest as CommandUpdateCategoryRequest,
};
use contracts::event_query::ListCategoriesRequest;
use contracts::event_query::event_query_service_client::EventQueryServiceClient;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    categories::models::{Category, CreateCategoryRequest, UpdateCategoryRequest},
    error::AppError,
};

pub async fn list(state: &AppState, user_id: Uuid) -> Result<Vec<Category>, AppError> {
    let mut client = query_client(state).await?;
    let reply = client
        .list_categories(ListCategoriesRequest {
            user_id: user_id.to_string(),
        })
        .await
        .map_err(map_status)?
        .into_inner();

    reply.items.into_iter().map(map_query_category).collect()
}

pub async fn create(
    state: &AppState,
    user_id: Uuid,
    payload: CreateCategoryRequest,
) -> Result<Category, AppError> {
    let mut client = command_client(state).await?;
    let reply = client
        .create_category(CommandCreateCategoryRequest {
            user_id: user_id.to_string(),
            name: payload.name,
            color: payload.color,
        })
        .await
        .map_err(map_status)?
        .into_inner();

    map_command_category(
        reply.category.ok_or_else(|| {
            AppError::Internal("Command response is missing category".to_string())
        })?,
    )
}

pub async fn update(
    state: &AppState,
    user_id: Uuid,
    category_id: Uuid,
    payload: UpdateCategoryRequest,
) -> Result<Category, AppError> {
    let mut client = command_client(state).await?;
    let reply = client
        .update_category(CommandUpdateCategoryRequest {
            user_id: user_id.to_string(),
            category_id: category_id.to_string(),
            name: payload.name,
            color: payload.color,
        })
        .await
        .map_err(map_status)?
        .into_inner();

    map_command_category(
        reply.category.ok_or_else(|| {
            AppError::Internal("Command response is missing category".to_string())
        })?,
    )
}

pub async fn delete(state: &AppState, user_id: Uuid, category_id: Uuid) -> Result<(), AppError> {
    let mut client = command_client(state).await?;
    client
        .delete_category(contracts::event_command::DeleteCategoryRequest {
            user_id: user_id.to_string(),
            category_id: category_id.to_string(),
        })
        .await
        .map_err(map_status)?;

    Ok(())
}

async fn command_client(
    state: &AppState,
) -> Result<EventCommandServiceClient<tonic::transport::Channel>, AppError> {
    EventCommandServiceClient::connect(state.config.event_command_service_url.clone())
        .await
        .map_err(|error| {
            AppError::Internal(format!("Event command service is unavailable: {error}"))
        })
}

async fn query_client(
    state: &AppState,
) -> Result<EventQueryServiceClient<tonic::transport::Channel>, AppError> {
    EventQueryServiceClient::connect(state.config.event_query_service_url.clone())
        .await
        .map_err(|error| AppError::Internal(format!("Event query service is unavailable: {error}")))
}

fn map_query_category(category: contracts::event_query::Category) -> Result<Category, AppError> {
    Ok(Category {
        id: parse_uuid(&category.id, "category id")?,
        user_id: parse_uuid(&category.user_id, "category user id")?,
        name: category.name,
        color: category.color,
        created_at: parse_timestamp(&category.created_at, "category created_at")?,
        updated_at: parse_timestamp(&category.updated_at, "category updated_at")?,
    })
}

fn map_command_category(
    category: contracts::event_command::Category,
) -> Result<Category, AppError> {
    Ok(Category {
        id: parse_uuid(&category.id, "category id")?,
        user_id: parse_uuid(&category.user_id, "category user id")?,
        name: category.name,
        color: category.color,
        created_at: parse_timestamp(&category.created_at, "category created_at")?,
        updated_at: parse_timestamp(&category.updated_at, "category updated_at")?,
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
        _ => AppError::Internal(format!("Category service error: {}", status.message())),
    }
}
