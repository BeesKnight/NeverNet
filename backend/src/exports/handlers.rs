use axum::{
    Json,
    extract::{Path, State},
    response::Response,
};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    error::AppError,
    exports::{
        models::{CreateExportRequest, ExportJob},
        service,
    },
    shared::{api::ApiResponse, auth::CurrentUser},
};

pub async fn list(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> Result<Json<ApiResponse<Vec<ExportJob>>>, AppError> {
    let exports = service::list(&state, current_user.user_id).await?;
    Ok(Json(ApiResponse::new(exports)))
}

pub async fn create(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(payload): Json<CreateExportRequest>,
) -> Result<Json<ApiResponse<ExportJob>>, AppError> {
    let job = service::create(&state, current_user.user_id, payload).await?;
    Ok(Json(ApiResponse::new(job)))
}

pub async fn get(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(export_id): Path<Uuid>,
) -> Result<Json<ApiResponse<ExportJob>>, AppError> {
    let job = service::get(&state, current_user.user_id, export_id).await?;
    Ok(Json(ApiResponse::new(job)))
}

pub async fn download(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(export_id): Path<Uuid>,
) -> Result<Response, AppError> {
    service::download(&state, current_user.user_id, export_id).await
}
