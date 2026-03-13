use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use shared_kernel::auth::{AUTH_COOKIE_NAME, decode_token};
use time::Duration;
use uuid::Uuid;

use crate::{app_state::AppState, error::AppError};

pub const CSRF_COOKIE_NAME: &str = "eventdesign_csrf";
pub const CSRF_HEADER_NAME: &str = "x-csrf-token";

#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub user_id: Uuid,
    pub token: String,
}

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);
        let token = jar
            .get(AUTH_COOKIE_NAME)
            .map(|cookie| cookie.value().to_string())
            .ok_or_else(|| AppError::Unauthorized("Missing auth session".to_string()))?;
        let claims = decode_token(&state.config.jwt_secret, &token)
            .map_err(|_| AppError::Unauthorized("Invalid or expired token".to_string()))?;
        let has_session = sqlx::query_scalar::<_, i32>(
            r#"
            SELECT 1
            FROM sessions
            WHERE id = $1
              AND user_id = $2
              AND revoked_at IS NULL
              AND expires_at > NOW()
            "#,
        )
        .bind(claims.sid)
        .bind(claims.sub)
        .fetch_optional(&state.pool)
        .await?;

        if has_session.is_none() {
            observability::increment_security_event("session_rejected");
            return Err(AppError::Unauthorized(
                "Invalid or expired token".to_string(),
            ));
        }

        Ok(CurrentUser {
            user_id: claims.sub,
            token,
        })
    }
}

pub fn build_auth_cookie(token: String, secure: bool) -> Cookie<'static> {
    Cookie::build((AUTH_COOKIE_NAME, token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(Duration::seconds(60 * 60 * 24 * 7))
        .secure(secure)
        .build()
}

pub fn build_removal_cookie(secure: bool) -> Cookie<'static> {
    Cookie::build((AUTH_COOKIE_NAME, ""))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(Duration::seconds(0))
        .secure(secure)
        .build()
}

pub fn build_csrf_cookie(token: String, secure: bool) -> Cookie<'static> {
    Cookie::build((CSRF_COOKIE_NAME, token))
        .path("/")
        .same_site(SameSite::Lax)
        .max_age(Duration::seconds(60 * 60 * 24 * 7))
        .secure(secure)
        .build()
}

pub fn build_csrf_removal_cookie(secure: bool) -> Cookie<'static> {
    Cookie::build((CSRF_COOKIE_NAME, ""))
        .path("/")
        .same_site(SameSite::Lax)
        .max_age(Duration::seconds(0))
        .secure(secure)
        .build()
}

pub fn generate_csrf_token() -> String {
    Uuid::new_v4().simple().to_string()
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::http::{Request, header::COOKIE};
    use persistence::connect_pool;
    use redis::Client;
    use shared_kernel::auth::create_token;

    use super::*;
    use crate::{app_state::AppState, config::Config};

    #[test]
    fn auth_cookie_is_http_only_and_lax() {
        let cookie = build_auth_cookie("session-token".to_string(), true);

        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
        assert_eq!(cookie.secure(), Some(true));
        assert_eq!(cookie.path(), Some("/"));
    }

    #[test]
    fn csrf_removal_cookie_clears_token() {
        let cookie = build_csrf_removal_cookie(false);

        assert_eq!(cookie.value(), "");
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
        assert_eq!(cookie.secure(), Some(false));
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.max_age(), Some(Duration::seconds(0)));
    }

    #[tokio::test]
    async fn current_user_accepts_valid_session_cookie() {
        let database_url = match std::env::var("DATABASE_URL") {
            Ok(value) => value,
            Err(_) => return,
        };
        let pool = connect_pool(&database_url, 1)
            .await
            .expect("test database should connect");
        sqlx::migrate!("../../migrations")
            .run(&pool)
            .await
            .expect("migrations should apply");
        let user_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO users (id, email, password_hash, full_name)
            VALUES ($1, 'edge-auth@eventdesign.local', 'hash', 'Edge Auth')
            "#,
        )
        .bind(user_id)
        .execute(&pool)
        .await
        .expect("user insert should succeed");
        sqlx::query(
            r#"
            INSERT INTO sessions (id, user_id, expires_at)
            VALUES ($1, $2, NOW() + INTERVAL '1 day')
            "#,
        )
        .bind(session_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .expect("session insert should succeed");

        let state = AppState::new(
            pool,
            Client::open("redis://127.0.0.1:6379").expect("redis client should build"),
            Arc::new(Config {
                database_url: "postgres://test".to_string(),
                jwt_secret: "test-secret".to_string(),
                redis_url: "redis://127.0.0.1:6379".to_string(),
                port: 0,
                metrics_port: 0,
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
        );
        let token =
            create_token(&state.config.jwt_secret, user_id, session_id).expect("token is valid");
        let request = Request::builder()
            .uri("/api/auth/me")
            .header(COOKIE, format!("{AUTH_COOKIE_NAME}={token}"))
            .body(())
            .expect("request should build");
        let (mut parts, _) = request.into_parts();

        let current_user = CurrentUser::from_request_parts(&mut parts, &state)
            .await
            .expect("session cookie should authenticate");

        assert_eq!(current_user.user_id, user_id);
        assert_eq!(current_user.token, token);
    }
}
