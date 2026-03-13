use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    Unauthorized(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    Config(String),
    #[error("{0}")]
    Internal(String),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    JwtEncode(#[from] jsonwebtoken::errors::Error),
}

#[derive(Serialize)]
struct ErrorBody {
    error: ErrorMessage,
}

#[derive(Serialize)]
struct ErrorMessage {
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Config(_) | AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Database(_)
            | AppError::Migration(_)
            | AppError::Io(_)
            | AppError::JwtEncode(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        if matches!(self, AppError::Database(_) | AppError::Migration(_)) {
            tracing::error!("database error: {}", self);
        }

        let body = ErrorBody {
            error: ErrorMessage {
                message: self.to_string(),
            },
        };

        (status, Json(body)).into_response()
    }
}

impl From<argon2::password_hash::Error> for AppError {
    fn from(error: argon2::password_hash::Error) -> Self {
        AppError::Internal(error.to_string())
    }
}

impl From<axum::http::Error> for AppError {
    fn from(error: axum::http::Error) -> Self {
        AppError::Internal(error.to_string())
    }
}

pub fn is_constraint(error: &sqlx::Error, constraint_name: &str) -> bool {
    match error {
        sqlx::Error::Database(database_error) => {
            database_error.constraint() == Some(constraint_name)
        }
        _ => false,
    }
}
