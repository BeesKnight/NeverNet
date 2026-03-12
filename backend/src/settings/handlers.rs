use axum::{Json, extract::State};

use crate::{
    app_state::AppState,
    error::AppError,
    settings::{
        models::{UiSettings, UpdateSettingsRequest},
        service,
    },
    shared::{api::ApiResponse, auth::CurrentUser},
};

pub async fn get(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> Result<Json<ApiResponse<UiSettings>>, AppError> {
    let settings = service::get(&state, current_user.user_id).await?;
    Ok(Json(ApiResponse::new(settings)))
}

pub async fn update(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(payload): Json<UpdateSettingsRequest>,
) -> Result<Json<ApiResponse<UiSettings>>, AppError> {
    let settings = service::update(&state, current_user.user_id, payload).await?;
    Ok(Json(ApiResponse::new(settings)))
}
