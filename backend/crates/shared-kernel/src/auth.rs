use std::time::{SystemTime, UNIX_EPOCH};

use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

pub const AUTH_COOKIE_NAME: &str = "eventdesign_session";
pub const TOKEN_TTL_SECONDS: usize = 60 * 60 * 24 * 7;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid or expired token")]
    InvalidToken,
    #[error("Could not issue auth token")]
    TokenCreationFailed,
}

pub fn create_token(secret: &str, user_id: Uuid) -> Result<String, AuthError> {
    let now = now_ts();
    let claims = Claims {
        sub: user_id,
        iat: now,
        exp: now + TOKEN_TTL_SECONDS,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| AuthError::TokenCreationFailed)
}

pub fn decode_token(secret: &str, token: &str) -> Result<Claims, AuthError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AuthError::InvalidToken)
}

fn now_ts() -> usize {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as usize
}
