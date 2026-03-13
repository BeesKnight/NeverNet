use axum::{
    Json,
    extract::{Query, State},
};

use crate::{
    app_state::AppState,
    error::AppError,
    events::models::{EventFilters, EventListItem},
    shared::{api::ApiResponse, auth::CurrentUser},
};

use super::service;

pub async fn list(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Query(filters): Query<EventFilters>,
) -> Result<Json<ApiResponse<Vec<EventListItem>>>, AppError> {
    let events = service::list(&state, current_user.user_id, filters).await?;
    Ok(Json(ApiResponse::new(events)))
}
