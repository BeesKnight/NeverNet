use axum::{
    Json,
    extract::{Query, State},
};

use crate::{
    app_state::AppState,
    calendar::models::{CalendarFilters, CalendarItem},
    error::AppError,
    shared::{api::ApiResponse, auth::CurrentUser},
};

use super::service;

pub async fn list(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Query(filters): Query<CalendarFilters>,
) -> Result<Json<ApiResponse<Vec<CalendarItem>>>, AppError> {
    let events = service::list(&state, current_user.user_id, filters).await?;
    Ok(Json(ApiResponse::new(events)))
}
