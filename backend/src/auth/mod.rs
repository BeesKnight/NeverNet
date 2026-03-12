mod handlers;
pub mod models;
mod repository;
pub mod service;
mod validation;

use axum::{
    Router,
    routing::{get, post},
};

use crate::app_state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(handlers::register))
        .route("/login", post(handlers::login))
        .route("/me", get(handlers::me))
}
