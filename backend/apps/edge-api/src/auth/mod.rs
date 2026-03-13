mod handlers;
pub mod models;
pub(crate) mod service;

use axum::{
    Router,
    routing::{get, post},
};

use crate::app_state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/csrf", get(handlers::csrf))
        .route("/register", post(handlers::register))
        .route("/login", post(handlers::login))
        .route("/logout", post(handlers::logout))
        .route("/me", get(handlers::me))
}
