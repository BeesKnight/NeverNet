use chrono::{DateTime, Utc};
use contracts::event_command::event_command_service_client::EventCommandServiceClient;
use contracts::event_command::{
    CreateCategoryRequest as CommandCreateCategoryRequest,
    UpdateCategoryRequest as CommandUpdateCategoryRequest,
};
use contracts::event_query::ListCategoriesRequest;
use contracts::event_query::event_query_service_client::EventQueryServiceClient;
use tonic::transport::Channel;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    categories::models::{Category, CreateCategoryRequest, UpdateCategoryRequest},
    error::AppError,
    shared::grpc,
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

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn sample_query_category() -> contracts::event_query::Category {
        contracts::event_query::Category {
            id: Uuid::new_v4().to_string(),
            user_id: Uuid::new_v4().to_string(),
            name: "Conference".to_string(),
            color: "#0f766e".to_string(),
            created_at: Utc
                .with_ymd_and_hms(2026, 3, 13, 10, 0, 0)
                .unwrap()
                .to_rfc3339(),
            updated_at: Utc
                .with_ymd_and_hms(2026, 3, 13, 10, 5, 0)
                .unwrap()
                .to_rfc3339(),
        }
    }

    #[test]
    fn maps_categories_from_query_and_command_services() {
        let query_category = map_query_category(sample_query_category()).expect("query category");
        let command_category = map_command_category(contracts::event_command::Category {
            id: query_category.id.to_string(),
            user_id: query_category.user_id.to_string(),
            name: query_category.name.clone(),
            color: query_category.color.clone(),
            created_at: query_category.created_at.to_rfc3339(),
            updated_at: query_category.updated_at.to_rfc3339(),
        })
        .expect("command category");

        assert_eq!(query_category.name, "Conference");
        assert_eq!(command_category.color, "#0f766e");
    }

    #[test]
    fn rejects_invalid_category_identifiers() {
        let mut category = sample_query_category();
        category.id = "invalid".to_string();

        assert!(map_query_category(category).is_err());
    }

    #[test]
    fn maps_category_status_codes() {
        assert!(matches!(
            map_status(tonic::Status::invalid_argument("bad")),
            AppError::BadRequest(_)
        ));
        assert!(matches!(
            map_status(tonic::Status::not_found("missing")),
            AppError::NotFound(_)
        ));
        assert!(matches!(
            map_status(tonic::Status::already_exists("exists")),
            AppError::Conflict(_)
        ));
        assert!(matches!(
            map_status(tonic::Status::internal("oops")),
            AppError::Internal(_)
        ));
    }
}
