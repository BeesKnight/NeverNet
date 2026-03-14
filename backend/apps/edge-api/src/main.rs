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
    let redis = redis::Client::open(config.redis_url.clone())
        .map_err(|error| AppError::Config(format!("Invalid REDIS_URL: {error}")))?;

    observability::spawn_metrics_server("edge-api", config.metrics_port);

    let state = AppState::new(redis, config.clone());
    let app = build_app(state)?;

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

fn build_app(state: AppState) -> Result<Router, AppError> {
    let origins = state
        .config
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

    Ok(Router::new()
        .route("/healthz", get(health_check))
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
        .with_state(state))
}

#[cfg(test)]
mod tests {
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use tower::util::ServiceExt;

    use super::*;

    fn state(frontend_origins: Vec<String>) -> AppState {
        AppState::new(
            redis::Client::open("redis://127.0.0.1:6379").expect("redis url should be valid"),
            Arc::new(Config {
                redis_url: "redis://127.0.0.1:6379".to_string(),
                port: 8080,
                metrics_port: 9100,
                identity_service_url: "http://127.0.0.1:50051".to_string(),
                event_command_service_url: "http://127.0.0.1:50052".to_string(),
                event_query_service_url: "http://127.0.0.1:50053".to_string(),
                report_service_url: "http://127.0.0.1:50054".to_string(),
                frontend_origins,
                auth_cookie_secure: false,
                rate_limit_window_seconds: 60,
                rate_limit_requests_per_window: 300,
                auth_rate_limit_requests_per_window: 20,
            }),
        )
    }

    #[tokio::test]
    async fn health_check_returns_ok_payload() {
        let response = health_check().await;

        assert_eq!(response.0.data, "ok");
    }

    #[tokio::test]
    async fn build_app_serves_health_and_known_routes() {
        let app = build_app(state(vec!["http://localhost:3000".to_string()]))
            .expect("router should build");

        let health_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let csrf_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/auth/csrf")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let categories_response = app
            .oneshot(
                Request::builder()
                    .uri("/api/categories")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let health_body = to_bytes(health_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let health_body = String::from_utf8(health_body.to_vec()).unwrap();

        assert!(health_body.contains("\"ok\""));
        assert_eq!(csrf_response.status(), StatusCode::OK);
        assert_eq!(categories_response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn build_app_rejects_invalid_frontend_header_values() {
        let error = build_app(state(vec![
            "http://localhost:3000\r\nx-test: 1".to_string(),
        ]))
        .expect_err("invalid header value should fail");

        assert!(error.to_string().contains("Invalid frontend origin"));
    }
}
