mod handlers;
pub mod models;
mod repository;
pub mod service;

use axum::{Router, routing::get};

use crate::app_state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::list).post(handlers::create))
        .route("/:id/download", get(handlers::download))
}
