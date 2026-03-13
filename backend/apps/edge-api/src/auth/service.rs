use chrono::{DateTime, Utc};
use contracts::identity::identity_service_client::IdentityServiceClient;
use contracts::identity::{
    CurrentUserRequest, LoginRequest as IdentityLoginRequest,
    RegisterRequest as IdentityRegisterRequest,
};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    auth::models::{LoginRequest, RegisterRequest},
    error::AppError,
    users::models::UserProfile,
};

pub struct AuthenticatedSession {
    pub token: String,
    pub user: UserProfile,
}

pub async fn register(
    state: &AppState,
    payload: RegisterRequest,
) -> Result<AuthenticatedSession, AppError> {
    let mut client = identity_client(state).await?;
    let reply = client
        .register(IdentityRegisterRequest {
            email: payload.email,
            password: payload.password,
            full_name: payload.full_name,
        })
        .await
        .map_err(map_identity_status)?
        .into_inner();

    Ok(AuthenticatedSession {
        token: reply.token,
        user: map_identity_user(reply.user)?,
    })
}

pub async fn login(
    state: &AppState,
    payload: LoginRequest,
) -> Result<AuthenticatedSession, AppError> {
    let mut client = identity_client(state).await?;
    let reply = client
        .login(IdentityLoginRequest {
            email: payload.email,
            password: payload.password,
        })
        .await
        .map_err(map_identity_status)?
        .into_inner();

    Ok(AuthenticatedSession {
        token: reply.token,
        user: map_identity_user(reply.user)?,
    })
}

pub async fn get_current_user(state: &AppState, token: &str) -> Result<UserProfile, AppError> {
    let mut client = identity_client(state).await?;
    let reply = client
        .get_current_user(CurrentUserRequest {
            token: token.to_string(),
        })
        .await
        .map_err(map_identity_status)?
        .into_inner();

    map_identity_user(reply.user)
}

async fn identity_client(
    state: &AppState,
) -> Result<IdentityServiceClient<tonic::transport::Channel>, AppError> {
    IdentityServiceClient::connect(state.config.identity_service_url.clone())
        .await
        .map_err(|error| AppError::Internal(format!("Identity service is unavailable: {error}")))
}

fn map_identity_user(user: Option<contracts::identity::User>) -> Result<UserProfile, AppError> {
    let user =
        user.ok_or_else(|| AppError::Internal("Identity response is missing a user".to_string()))?;
    let created_at = DateTime::parse_from_rfc3339(&user.created_at)
        .map_err(|_| {
            AppError::Internal("Identity response contains an invalid timestamp".to_string())
        })?
        .with_timezone(&Utc);
    let id = Uuid::parse_str(&user.id).map_err(|_| {
        AppError::Internal("Identity response contains an invalid user id".to_string())
    })?;

    Ok(UserProfile {
        id,
        email: user.email,
        full_name: user.full_name,
        created_at,
    })
}

fn map_identity_status(status: tonic::Status) -> AppError {
    match status.code() {
        tonic::Code::InvalidArgument => AppError::BadRequest(status.message().to_string()),
        tonic::Code::Unauthenticated => AppError::Unauthorized(status.message().to_string()),
        tonic::Code::NotFound => AppError::NotFound(status.message().to_string()),
        tonic::Code::AlreadyExists => AppError::Conflict(status.message().to_string()),
        _ => AppError::Internal(format!("Identity service error: {}", status.message())),
    }
}
