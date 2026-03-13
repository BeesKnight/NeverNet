mod app_state;
mod auth;
mod calendar;
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

use axum::{
    Router,
    http::{HeaderValue, Method, header},
    routing::get,
};
use persistence::connect_pool;
use tokio::net::TcpListener;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    trace::TraceLayer,
};

use crate::{app_state::AppState, config::Config, error::AppError, shared::api::ApiResponse};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    observability::init_tracing("edge_api=info,tower_http=info");

    let config = Arc::new(Config::from_env()?);
    let pool = connect_pool(&config.database_url, 10).await?;

    sqlx::migrate!("../../migrations").run(&pool).await?;

    let state = AppState::new(pool, config.clone());
    exports::service::resume_pending_jobs(state.clone()).await;

    let origins = config
        .frontend_origins
        .iter()
        .map(|origin| {
            HeaderValue::from_str(origin)
                .map_err(|_| AppError::Config(format!("Invalid frontend origin: {origin}")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_credentials(true)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT]);

    let app = Router::new()
        .route("/health", get(health_check))
        .nest("/api/auth", auth::router())
        .nest("/api/calendar", calendar::router())
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

    tracing::info!("edge-api listening on http://{}", address);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> axum::Json<ApiResponse<&'static str>> {
    axum::Json(ApiResponse::new("ok"))
}
