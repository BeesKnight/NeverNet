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
