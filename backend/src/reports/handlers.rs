use axum::{
    Json,
    extract::{Query, State},
};

use crate::{
    app_state::AppState,
    error::AppError,
    events::models::EventFilters,
    reports::{models::ReportSummary, service},
    shared::{api::ApiResponse, auth::CurrentUser},
};

pub async fn summary(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Query(filters): Query<EventFilters>,
) -> Result<Json<ApiResponse<ReportSummary>>, AppError> {
    let report = service::generate_summary(&state, current_user.user_id, filters).await?;
    Ok(Json(ApiResponse::new(report)))
}
