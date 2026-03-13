use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};
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
            .or_else(|| bearer_token(parts))
            .ok_or_else(|| AppError::Unauthorized("Missing auth session".to_string()))?;
        let claims = decode_token(&state.config.jwt_secret, &token)
            .map_err(|_| AppError::Unauthorized("Invalid or expired token".to_string()))?;
        let has_session = sqlx::query_scalar::<_, i64>(
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

pub fn generate_csrf_token() -> String {
    Uuid::new_v4().simple().to_string()
}

fn bearer_token(parts: &Parts) -> Option<String> {
    parts
        .headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "))
        .map(ToOwned::to_owned)
}
