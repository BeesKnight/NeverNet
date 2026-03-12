mod app_state;
mod auth;
mod categories;
mod config;
mod error;
mod events;
mod exports;
mod reports;
mod settings;
mod shared;
mod users;

use std::{net::SocketAddr, sync::Arc};

use axum::{Router, routing::get};
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{app_state::AppState, config::Config, error::AppError, shared::api::ApiResponse};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "backend=info,tower_http=info".to_string()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Arc::new(Config::from_env()?);
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let state = AppState::new(pool, config.clone());
    exports::service::resume_pending_jobs(state.clone()).await;

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(health_check))
        .nest("/api/auth", auth::router())
        .nest("/api/categories", categories::router())
        .nest("/api/events", events::router())
        .nest("/api/reports", reports::router())
        .nest("/api/settings", settings::router())
        .nest("/api/exports", exports::router())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let address = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = TcpListener::bind(address).await?;

    tracing::info!("backend listening on http://{}", address);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> axum::Json<ApiResponse<&'static str>> {
    axum::Json(ApiResponse::new("ok"))
}
