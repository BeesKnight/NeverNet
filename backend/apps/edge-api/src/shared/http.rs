use std::{net::SocketAddr, time::SystemTime};

use axum::{
    body::Body,
    extract::{ConnectInfo, MatchedPath, Request, State},
    http::{Method, StatusCode},
    middleware::Next,
    response::Response,
};
use axum_extra::extract::cookie::CookieJar;
use redis::AsyncCommands;

use crate::{
    app_state::AppState,
    error::AppError,
    shared::auth::{CSRF_COOKIE_NAME, CSRF_HEADER_NAME},
};

pub async fn metrics_middleware(request: Request<Body>, next: Next) -> Response {
    let method = request.method().as_str().to_string();
    let route = request
        .extensions()
        .get::<MatchedPath>()
        .map(MatchedPath::as_str)
        .unwrap_or_else(|| request.uri().path())
        .to_string();
    let started_at = SystemTime::now();

    let response = next.run(request).await;
    let elapsed = started_at.elapsed().unwrap_or_default();

    observability::observe_http_request(&method, &route, response.status().as_u16(), elapsed);
    response
}

pub async fn csrf_middleware(request: Request<Body>, next: Next) -> Result<Response, AppError> {
    if is_safe_method(request.method()) || request.uri().path() == "/api/auth/csrf" {
        return Ok(next.run(request).await);
    }

    let jar = CookieJar::from_headers(request.headers());
    let cookie_token = jar
        .get(CSRF_COOKIE_NAME)
        .map(|cookie| cookie.value().to_string());
    let header_token = request
        .headers()
        .get(CSRF_HEADER_NAME)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    if matches!((&cookie_token, &header_token), (Some(cookie), Some(header)) if cookie == header) {
        return Ok(next.run(request).await);
    }

    observability::increment_security_event("csrf_rejected");
    Err(AppError::Unauthorized(
        "CSRF token is missing or invalid".to_string(),
    ))
}

pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let Some(client_id) = client_identifier(&request) else {
        return Ok(next.run(request).await);
    };

    let (scope, max_requests) = if request.uri().path().starts_with("/api/auth/") {
        ("auth", state.config.auth_rate_limit_requests_per_window)
    } else {
        ("api", state.config.rate_limit_requests_per_window)
    };

    match check_rate_limit(&state, &client_id, scope, max_requests).await {
        Ok(true) => Ok(next.run(request).await),
        Ok(false) => {
            observability::increment_security_event("rate_limited");
            Err(AppError::RateLimited(
                "Too many requests. Please retry shortly.".to_string(),
            ))
        }
        Err(error) => {
            tracing::warn!("rate limiter unavailable, allowing request: {error}");
            Ok(next.run(request).await)
        }
    }
}

fn is_safe_method(method: &Method) -> bool {
    matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS)
}

fn client_identifier(request: &Request<Body>) -> Option<String> {
    if let Some(value) = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return Some(value.to_string());
    }

    request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|connect_info| connect_info.0.ip().to_string())
}

async fn check_rate_limit(
    state: &AppState,
    client_id: &str,
    scope: &str,
    max_requests: u64,
) -> Result<bool, redis::RedisError> {
    let bucket = unix_time_bucket(state.config.rate_limit_window_seconds);
    let key = format!("rate-limit:{scope}:{client_id}:{bucket}");
    let mut connection = state.redis.get_multiplexed_tokio_connection().await?;
    let current: u64 = connection.incr(&key, 1_u8).await?;

    if current == 1 {
        let _: () = connection
            .expire(&key, state.config.rate_limit_window_seconds as i64)
            .await?;
    }

    Ok(current <= max_requests)
}

fn unix_time_bucket(window_seconds: u64) -> u64 {
    if window_seconds == 0 {
        return 0;
    }

    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        / window_seconds
}

#[allow(dead_code)]
fn _status_text(status: StatusCode) -> &'static str {
    status.canonical_reason().unwrap_or("unknown")
}
