mod handlers;
pub mod models;
pub mod repository;
mod service;
mod validation;

use axum::{
    Router,
    routing::{get, put},
};

use crate::app_state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::list).post(handlers::create))
        .route(
            "/{id}",
            put(handlers::update)
                .patch(handlers::update)
                .delete(handlers::delete),
        )
}
