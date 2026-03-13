use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use rand_core::OsRng;
use shared_kernel::auth::{create_token, decode_token};
use crate::{
    app_state::AppState,
    auth::{
        models::{AuthResponse, LoginRequest, RegisterRequest},
        repository, validation,
    },
    error::AppError,
    users::{models::UserProfile, repository as users_repository},
};

pub async fn register(
    state: &AppState,
    payload: RegisterRequest,
) -> Result<AuthResponse, AppError> {
    validation::validate_register(&payload)?;

    let email = payload.email.trim().to_lowercase();
    if users_repository::find_by_email(&state.pool, &email)
        .await?
        .is_some()
    {
        return Err(AppError::Conflict(
            "Email is already registered".to_string(),
        ));
    }

    let password_hash = hash_password(&payload.password)?;
    let user = users_repository::create_user(
        &state.pool,
        &email,
        &password_hash,
        payload.full_name.trim(),
    )
    .await
    .map_err(|error| {
        if crate::error::is_constraint(&error, "users_email_key") {
            AppError::Conflict("Email is already registered".to_string())
        } else {
            AppError::from(error)
        }
    })?;

    repository::create_default_settings(&state.pool, user.id).await?;
    let session = repository::create_session(&state.pool, user.id).await?;

    Ok(AuthResponse {
        token: create_token(&state.config.jwt_secret, user.id, session.id)
            .map_err(|_| AppError::Internal("Could not issue auth token".to_string()))?,
        user: UserProfile::from(user),
    })
}

pub async fn login(state: &AppState, payload: LoginRequest) -> Result<AuthResponse, AppError> {
    validation::validate_login(&payload)?;

    let email = payload.email.trim().to_lowercase();
    let user = users_repository::find_by_email(&state.pool, &email)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid email or password".to_string()))?;

    verify_password(&payload.password, &user.password_hash)?;
    let session = repository::create_session(&state.pool, user.id).await?;

    Ok(AuthResponse {
        token: create_token(&state.config.jwt_secret, user.id, session.id)
            .map_err(|_| AppError::Internal("Could not issue auth token".to_string()))?,
        user: UserProfile::from(user),
    })
}

pub async fn get_current_user(state: &AppState, token: &str) -> Result<UserProfile, AppError> {
    let claims = decode_token(&state.config.jwt_secret, token)
        .map_err(|_| AppError::Unauthorized("Invalid or expired token".to_string()))?;
    repository::find_active_session(&state.pool, claims.sid, claims.sub)
        .await?
        .ok_or_else(|| AppError::Unauthorized("User session is no longer valid".to_string()))?;

    let user = users_repository::find_by_id(&state.pool, claims.sub)
        .await?
        .ok_or_else(|| AppError::Unauthorized("User session is no longer valid".to_string()))?;

    Ok(user.into())
}

pub async fn logout(state: &AppState, token: &str) -> Result<(), AppError> {
    let claims = decode_token(&state.config.jwt_secret, token)
        .map_err(|_| AppError::Unauthorized("Invalid or expired token".to_string()))?;
    let revoked = repository::revoke_session(&state.pool, claims.sid, claims.sub).await?;

    if revoked == 0 {
        return Err(AppError::Unauthorized(
            "User session is no longer valid".to_string(),
        ));
    }

    Ok(())
}
fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    Ok(Argon2::default()
        .hash_password(password.as_bytes(), &salt)?
        .to_string())
}

fn verify_password(password: &str, password_hash: &str) -> Result<(), AppError> {
    let parsed_hash = PasswordHash::new(password_hash)?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::Unauthorized("Invalid email or password".to_string()))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use sqlx::PgPool;

    use super::*;
    use crate::config::Config;

    #[sqlx::test(migrations = "../../migrations")]
    async fn register_login_logout_session_flow(pool: PgPool) {
        let state = AppState::new(
            pool,
            Arc::new(Config {
                database_url: "postgres://test".to_string(),
                jwt_secret: "test-secret".to_string(),
                grpc_port: 0,
                metrics_port: 0,
            }),
        );

        let registered = register(
            &state,
            RegisterRequest {
                email: "demo@eventdesign.local".to_string(),
                password: "DemoPass123!".to_string(),
                full_name: "Defense Demo".to_string(),
            },
        )
        .await
        .expect("registration succeeds");

        let logged_in = login(
            &state,
            LoginRequest {
                email: "demo@eventdesign.local".to_string(),
                password: "DemoPass123!".to_string(),
            },
        )
        .await
        .expect("login succeeds");

        let current = get_current_user(&state, &registered.token)
            .await
            .expect("registered session stays valid");
        assert_eq!(current.email, "demo@eventdesign.local");

        logout(&state, &logged_in.token)
            .await
            .expect("logout revokes the session");
        let error = get_current_user(&state, &logged_in.token)
            .await
            .expect_err("revoked session should fail");

        assert!(matches!(error, AppError::Unauthorized(_)));
    }
}
