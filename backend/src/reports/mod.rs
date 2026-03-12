mod handlers;
pub mod models;
pub mod service;

use axum::{Router, routing::get};

use crate::app_state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/summary", get(handlers::summary))
}
