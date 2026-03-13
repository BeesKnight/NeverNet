mod app_state;
mod auth;
mod calendar;
mod categories;
mod config;
mod dashboard;
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
    extract::MatchedPath,
    http::{HeaderValue, Method, header},
    middleware,
    routing::get,
};
use persistence::connect_pool;
use tokio::net::TcpListener;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::{DefaultOnFailure, DefaultOnResponse, TraceLayer},
};
use tracing::Level;

use crate::{
    app_state::AppState,
    config::Config,
    error::AppError,
    shared::{
        api::ApiResponse,
        http::{csrf_middleware, metrics_middleware, rate_limit_middleware},
        request_context::with_request_context,
    },
};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    observability::init_tracing("edge-api", "edge_api=info,tower_http=info");

    let config = Arc::new(Config::from_env()?);
    let pool = connect_pool(&config.database_url, 10).await?;
    let redis = redis::Client::open(config.redis_url.clone())
        .map_err(|error| AppError::Config(format!("Invalid REDIS_URL: {error}")))?;

    observability::spawn_metrics_server("edge-api", config.metrics_port);

    let state = AppState::new(pool, redis, config.clone());

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
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
            header::HeaderName::from_static("x-csrf-token"),
            header::HeaderName::from_static("x-request-id"),
        ])
        .expose_headers([header::HeaderName::from_static("x-request-id")]);

    let trace = TraceLayer::new_for_http()
        .make_span_with(|request: &axum::http::Request<_>| {
            let request_id = request
                .headers()
                .get("x-request-id")
                .and_then(|value| value.to_str().ok())
                .unwrap_or("missing");
            let route = request
                .extensions()
                .get::<MatchedPath>()
                .map(MatchedPath::as_str)
                .unwrap_or_else(|| request.uri().path());

            tracing::info_span!(
                "http_request",
                method = %request.method(),
                route,
                request_id
            )
        })
        .on_response(DefaultOnResponse::new().level(Level::INFO))
        .on_failure(DefaultOnFailure::new().level(Level::ERROR));

    let app = Router::new()
        .route("/health", get(health_check))
        .nest("/api/auth", auth::router())
        .nest("/api/calendar", calendar::router())
        .nest("/api/categories", categories::router())
        .nest("/api/dashboard", dashboard::router())
        .nest("/api/events", events::router())
        .nest("/api/reports", reports::router())
        .nest("/api/settings", settings::router())
        .nest("/api/exports", exports::router())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            rate_limit_middleware,
        ))
        .layer(middleware::from_fn(csrf_middleware))
        .layer(middleware::from_fn(metrics_middleware))
        .layer(middleware::from_fn(with_request_context))
        .layer(PropagateRequestIdLayer::new(
            header::HeaderName::from_static("x-request-id"),
        ))
        .layer(SetRequestIdLayer::new(
            header::HeaderName::from_static("x-request-id"),
            MakeRequestUuid,
        ))
        .layer(cors)
        .layer(trace)
        .with_state(state);

    let address = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = TcpListener::bind(address).await?;

    tracing::info!("edge-api listening on http://{}", address);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

async fn health_check() -> axum::Json<ApiResponse<&'static str>> {
    axum::Json(ApiResponse::new("ok"))
}
