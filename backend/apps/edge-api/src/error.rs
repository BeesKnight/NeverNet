use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

use crate::shared::request_context;

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
    RateLimited(String),
    #[error("{0}")]
    Config(String),
    #[error("{0}")]
    Internal(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Serialize)]
struct ErrorBody {
    error: ErrorMessage,
}

#[derive(Serialize)]
struct ErrorMessage {
    code: &'static str,
    message: String,
    request_id: Option<String>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::RateLimited(_) => StatusCode::TOO_MANY_REQUESTS,
            AppError::Config(_) | AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        if matches!(
            self,
            AppError::Config(_) | AppError::Internal(_) | AppError::Io(_)
        ) {
            tracing::error!(
                request_id = request_context::current_request_id()
                    .as_deref()
                    .unwrap_or("missing"),
                "edge-api error: {}",
                self
            );
        }

        let body = ErrorBody {
            error: ErrorMessage {
                code: error_code(&self),
                message: self.to_string(),
                request_id: request_context::current_request_id(),
            },
        };

        (status, Json(body)).into_response()
    }
}

impl From<axum::http::Error> for AppError {
    fn from(error: axum::http::Error) -> Self {
        AppError::Internal(error.to_string())
    }
}

fn error_code(error: &AppError) -> &'static str {
    match error {
        AppError::BadRequest(_) => "bad_request",
        AppError::Unauthorized(_) => "unauthorized",
        AppError::NotFound(_) => "not_found",
        AppError::Conflict(_) => "conflict",
        AppError::RateLimited(_) => "rate_limited",
        AppError::Config(_) => "config_error",
        AppError::Internal(_) | AppError::Io(_) => "internal_error",
    }
}

#[cfg(test)]
mod tests {
    use axum::body::to_bytes;

    use super::*;

    #[tokio::test]
    async fn converts_errors_into_response_envelopes() {
        let response = AppError::Conflict("already exists".to_string()).into_response();
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body should be readable");
        let body = String::from_utf8(body.to_vec()).expect("body should be utf8");

        assert_eq!(status, StatusCode::CONFLICT);
        assert!(body.contains("\"code\":\"conflict\""));
        assert!(body.contains("already exists"));
    }

    #[test]
    fn maps_http_errors_to_internal_app_error() {
        let invalid_uri = "[".parse::<axum::http::Uri>().expect_err("uri should fail");
        let error = AppError::from(axum::http::Error::from(invalid_uri));

        assert!(matches!(error, AppError::Internal(_)));
    }
}
