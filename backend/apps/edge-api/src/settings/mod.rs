mod handlers;
pub mod models;
mod repository;
mod service;

use axum::{Router, routing::get};

use crate::app_state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route(
        "/",
        get(handlers::get)
            .put(handlers::update)
            .patch(handlers::update),
    )
}
