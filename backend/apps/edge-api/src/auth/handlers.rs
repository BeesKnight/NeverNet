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
        auth::{CurrentUser, build_auth_cookie, build_removal_cookie},
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

pub async fn me(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> Result<Json<ApiResponse<SessionResponse>>, AppError> {
    let user = service::get_current_user(&state, &current_user.token).await?;
    Ok(Json(ApiResponse::new(SessionResponse { user })))
}

pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> (CookieJar, Json<ApiResponse<&'static str>>) {
    let jar = jar.remove(build_removal_cookie(state.config.auth_cookie_secure));
    (jar, Json(ApiResponse::new("logged_out")))
}
