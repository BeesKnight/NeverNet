use axum::{Json, extract::State};

use crate::{
    app_state::AppState,
    auth::{
        models::{AuthResponse, LoginRequest, RegisterRequest},
        service,
    },
    error::AppError,
    shared::{api::ApiResponse, auth::CurrentUser},
    users::models::UserProfile,
};

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<ApiResponse<AuthResponse>>, AppError> {
    let response = service::register(&state, payload).await?;
    Ok(Json(ApiResponse::new(response)))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<ApiResponse<AuthResponse>>, AppError> {
    let response = service::login(&state, payload).await?;
    Ok(Json(ApiResponse::new(response)))
}

pub async fn me(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> Result<Json<ApiResponse<UserProfile>>, AppError> {
    let user = service::get_current_user(&state, current_user.user_id).await?;
    Ok(Json(ApiResponse::new(user)))
}

pub async fn logout(_current_user: CurrentUser) -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse::new("logged_out"))
}
