use chrono::{DateTime, Utc};
use contracts::identity::identity_service_client::IdentityServiceClient;
use contracts::identity::{
    CurrentUserRequest, LoginRequest as IdentityLoginRequest, LogoutRequest,
    RegisterRequest as IdentityRegisterRequest,
};
use tonic::transport::Channel;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    auth::models::{LoginRequest, RegisterRequest},
    error::AppError,
    shared::grpc,
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

pub async fn logout(state: &AppState, token: &str) -> Result<(), AppError> {
    let mut client = identity_client(state).await?;
    client
        .logout(LogoutRequest {
            token: token.to_string(),
        })
        .await
        .map_err(map_identity_status)?;

    Ok(())
}

async fn identity_client(
    state: &AppState,
) -> Result<
    IdentityServiceClient<
        tonic::service::interceptor::InterceptedService<Channel, grpc::RequestIdInterceptor>,
    >,
    AppError,
> {
    let channel =
        grpc::connect_channel(&state.config.identity_service_url, "Identity service").await?;

    Ok(IdentityServiceClient::with_interceptor(
        channel,
        grpc::RequestIdInterceptor,
    ))
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

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn sample_identity_user() -> contracts::identity::User {
        contracts::identity::User {
            id: Uuid::new_v4().to_string(),
            email: "demo@eventdesign.local".to_string(),
            full_name: "Demo User".to_string(),
            created_at: Utc
                .with_ymd_and_hms(2026, 3, 13, 10, 0, 0)
                .unwrap()
                .to_rfc3339(),
        }
    }

    #[test]
    fn maps_identity_user_payload() {
        let user = map_identity_user(Some(sample_identity_user())).expect("user should map");

        assert_eq!(user.email, "demo@eventdesign.local");
        assert_eq!(user.full_name, "Demo User");
    }

    #[test]
    fn rejects_invalid_identity_user_payload() {
        let mut user = sample_identity_user();
        user.id = "invalid".to_string();

        assert!(map_identity_user(None).is_err());
        assert!(map_identity_user(Some(user)).is_err());
    }

    #[test]
    fn maps_identity_status_codes() {
        assert!(matches!(
            map_identity_status(tonic::Status::invalid_argument("bad")),
            AppError::BadRequest(_)
        ));
        assert!(matches!(
            map_identity_status(tonic::Status::unauthenticated("denied")),
            AppError::Unauthorized(_)
        ));
        assert!(matches!(
            map_identity_status(tonic::Status::already_exists("exists")),
            AppError::Conflict(_)
        ));
        assert!(matches!(
            map_identity_status(tonic::Status::internal("oops")),
            AppError::Internal(_)
        ));
    }
}
