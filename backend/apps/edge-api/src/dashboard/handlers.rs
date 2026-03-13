use axum::{Json, extract::State};

use crate::{
    app_state::AppState,
    dashboard::{models::DashboardResponse, service},
    error::AppError,
    shared::{api::ApiResponse, auth::CurrentUser},
};

pub async fn get(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> Result<Json<ApiResponse<DashboardResponse>>, AppError> {
    let dashboard = service::get(&state, current_user.user_id).await?;
    Ok(Json(ApiResponse::new(dashboard)))
}
