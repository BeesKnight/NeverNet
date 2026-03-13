use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use shared_kernel::auth::AUTH_COOKIE_NAME;
use time::Duration;
use uuid::Uuid;

use crate::{app_state::AppState, auth::service, error::AppError, users::models::UserProfile};

pub const CSRF_COOKIE_NAME: &str = "eventdesign_csrf";
pub const CSRF_HEADER_NAME: &str = "x-csrf-token";

#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub user_id: Uuid,
    pub token: String,
    pub user: UserProfile,
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
        let user = match service::get_current_user(state, &token).await {
            Ok(user) => user,
            Err(error) => {
                if matches!(error, AppError::Unauthorized(_)) {
                    observability::increment_security_event("session_rejected");
                }
                return Err(error);
            }
        };

        Ok(CurrentUser {
            user_id: user.id,
            token,
            user,
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
    use super::*;

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
}
