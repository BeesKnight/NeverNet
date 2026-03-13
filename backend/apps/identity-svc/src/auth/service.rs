use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use rand_core::OsRng;
use shared_kernel::auth::create_token;
use uuid::Uuid;

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

    Ok(AuthResponse {
        token: create_token(&state.config.jwt_secret, user.id)
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

    Ok(AuthResponse {
        token: create_token(&state.config.jwt_secret, user.id)
            .map_err(|_| AppError::Internal("Could not issue auth token".to_string()))?,
        user: UserProfile::from(user),
    })
}

pub async fn get_current_user(state: &AppState, user_id: Uuid) -> Result<UserProfile, AppError> {
    let user = users_repository::find_by_id(&state.pool, user_id)
        .await?
        .ok_or_else(|| AppError::Unauthorized("User session is no longer valid".to_string()))?;

    Ok(user.into())
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
