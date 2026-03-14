use axum::{Json, extract::State};
use axum_extra::extract::cookie::CookieJar;

use crate::{
    app_state::AppState,
    auth::{
        models::{LoginRequest, RegisterRequest, SessionResponse},
        service,
    },
    error::AppError,
    shared::{
        api::ApiResponse,
        auth::{
            CurrentUser, build_auth_cookie, build_csrf_cookie, build_csrf_removal_cookie,
            build_removal_cookie, generate_csrf_token,
        },
    },
};

pub async fn register(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<RegisterRequest>,
) -> Result<(CookieJar, Json<ApiResponse<SessionResponse>>), AppError> {
    let response = service::register(&state, payload).await?;
    let jar = jar.add(build_auth_cookie(
        response.token,
        state.config.auth_cookie_secure,
    ));

    Ok((
        jar,
        Json(ApiResponse::new(SessionResponse {
            user: response.user,
        })),
    ))
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<LoginRequest>,
) -> Result<(CookieJar, Json<ApiResponse<SessionResponse>>), AppError> {
    let response = service::login(&state, payload).await?;
    let jar = jar.add(build_auth_cookie(
        response.token,
        state.config.auth_cookie_secure,
    ));

    Ok((
        jar,
        Json(ApiResponse::new(SessionResponse {
            user: response.user,
        })),
    ))
}

pub async fn me(current_user: CurrentUser) -> Result<Json<ApiResponse<SessionResponse>>, AppError> {
    Ok(Json(ApiResponse::new(SessionResponse {
        user: current_user.user,
    })))
}

pub async fn csrf(
    State(state): State<AppState>,
    jar: CookieJar,
) -> (CookieJar, Json<ApiResponse<serde_json::Value>>) {
    let token = generate_csrf_token();
    let jar = jar.add(build_csrf_cookie(
        token.clone(),
        state.config.auth_cookie_secure,
    ));

    (
        jar,
        Json(ApiResponse::new(serde_json::json!({
            "csrf_token": token,
        }))),
    )
}

pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
    current_user: Result<CurrentUser, AppError>,
) -> Result<(CookieJar, Json<ApiResponse<&'static str>>), AppError> {
    if let Ok(current_user) = current_user {
        service::logout(&state, &current_user.token).await?;
    }

    let jar = jar
        .remove(build_removal_cookie(state.config.auth_cookie_secure))
        .remove(build_csrf_removal_cookie(state.config.auth_cookie_secure));
    Ok((jar, Json(ApiResponse::new("logged_out"))))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::Utc;
    use uuid::Uuid;

    use super::*;
    use crate::{
        config::Config,
        shared::auth::{CSRF_COOKIE_NAME, build_auth_cookie, build_csrf_cookie},
        users::models::UserProfile,
    };

    fn state() -> AppState {
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
                frontend_origins: vec!["http://localhost:3000".to_string()],
                auth_cookie_secure: false,
                rate_limit_window_seconds: 60,
                rate_limit_requests_per_window: 300,
                auth_rate_limit_requests_per_window: 20,
            }),
        )
    }

    fn current_user() -> CurrentUser {
        CurrentUser {
            user_id: Uuid::new_v4(),
            token: "session-token".to_string(),
            user: UserProfile {
                id: Uuid::new_v4(),
                email: "user@eventdesign.local".to_string(),
                full_name: "Edge User".to_string(),
                created_at: Utc::now(),
            },
        }
    }

    #[tokio::test]
    async fn me_returns_current_user_payload() {
        let response = me(current_user()).await.expect("me should succeed");

        assert_eq!(response.0.data.user.email, "user@eventdesign.local");
    }

    #[tokio::test]
    async fn csrf_sets_cookie_and_returns_matching_token() {
        let (jar, response) = csrf(State(state()), CookieJar::new()).await;
        let cookie = jar
            .get(CSRF_COOKIE_NAME)
            .expect("csrf cookie should be set");

        assert_eq!(response.0.data["csrf_token"], cookie.value());
    }

    #[tokio::test]
    async fn logout_clears_auth_and_csrf_cookies_without_remote_call() {
        let jar = CookieJar::new()
            .add(build_auth_cookie("session-token".to_string(), false))
            .add(build_csrf_cookie("csrf-token".to_string(), false));

        let (jar, response) = logout(
            State(state()),
            jar,
            Err(AppError::Unauthorized("missing session".to_string())),
        )
        .await
        .expect("logout should succeed without current session");

        assert_eq!(response.0.data, "logged_out");
        assert!(jar.get(shared_kernel::auth::AUTH_COOKIE_NAME).is_none());
        assert!(jar.get(CSRF_COOKIE_NAME).is_none());
    }
}
