use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    categories::{
        models::{Category, CreateCategoryRequest, UpdateCategoryRequest},
        service,
    },
    error::AppError,
    shared::{api::ApiResponse, auth::CurrentUser},
};

pub async fn list(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> Result<Json<ApiResponse<Vec<Category>>>, AppError> {
    let categories = service::list(&state, current_user.user_id).await?;
    Ok(Json(ApiResponse::new(categories)))
}

pub async fn create(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(payload): Json<CreateCategoryRequest>,
) -> Result<Json<ApiResponse<Category>>, AppError> {
    let category = service::create(&state, current_user.user_id, payload).await?;
    Ok(Json(ApiResponse::new(category)))
}

pub async fn update(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(category_id): Path<Uuid>,
    Json(payload): Json<UpdateCategoryRequest>,
) -> Result<Json<ApiResponse<Category>>, AppError> {
    let category = service::update(&state, current_user.user_id, category_id, payload).await?;
    Ok(Json(ApiResponse::new(category)))
}

pub async fn delete(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(category_id): Path<Uuid>,
) -> Result<Json<ApiResponse<&'static str>>, AppError> {
    service::delete(&state, current_user.user_id, category_id).await?;
    Ok(Json(ApiResponse::new("deleted")))
}
