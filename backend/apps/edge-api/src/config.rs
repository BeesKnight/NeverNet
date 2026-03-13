use std::env;

use crate::error::AppError;

#[derive(Debug)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub redis_url: String,
    pub port: u16,
    pub metrics_port: u16,
    pub identity_service_url: String,
    pub event_command_service_url: String,
    pub event_query_service_url: String,
    pub report_service_url: String,
    pub frontend_origins: Vec<String>,
    pub auth_cookie_secure: bool,
    pub rate_limit_window_seconds: u64,
    pub rate_limit_requests_per_window: u64,
    pub auth_rate_limit_requests_per_window: u64,
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        dotenvy::dotenv().ok();

        let database_url = required("DATABASE_URL")?;
        let jwt_secret = required("JWT_SECRET")?;
        let redis_url =
            env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
        let port = env::var("PORT")
            .ok()
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(8080);
        let metrics_port = env::var("METRICS_PORT")
            .ok()
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(9100);
        let identity_service_url = env::var("IDENTITY_SERVICE_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:50051".to_string());
        let event_command_service_url = env::var("EVENT_COMMAND_SERVICE_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:50052".to_string());
        let event_query_service_url = env::var("EVENT_QUERY_SERVICE_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:50053".to_string());
        let report_service_url =
            env::var("REPORT_SERVICE_URL").unwrap_or_else(|_| "http://127.0.0.1:50054".to_string());
        let frontend_origins: Vec<String> = env::var("FRONTEND_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3000,http://localhost:5173".to_string())
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .collect();
        let auth_cookie_secure = env::var("AUTH_COOKIE_SECURE")
            .ok()
            .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "True"))
            .unwrap_or(false);
        let rate_limit_window_seconds = env::var("RATE_LIMIT_WINDOW_SECONDS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(60);
        let rate_limit_requests_per_window = env::var("RATE_LIMIT_REQUESTS_PER_WINDOW")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(300);
        let auth_rate_limit_requests_per_window = env::var("AUTH_RATE_LIMIT_REQUESTS_PER_WINDOW")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(20);

        if frontend_origins.is_empty() {
            return Err(AppError::Config(
                "FRONTEND_ORIGINS must contain at least one origin".to_string(),
            ));
        }

        Ok(Self {
            database_url,
            jwt_secret,
            redis_url,
            port,
            metrics_port,
            identity_service_url,
            event_command_service_url,
            event_query_service_url,
            report_service_url,
            frontend_origins,
            auth_cookie_secure,
            rate_limit_window_seconds,
            rate_limit_requests_per_window,
            auth_rate_limit_requests_per_window,
        })
    }
}

fn required(key: &str) -> Result<String, AppError> {
    env::var(key).map_err(|_| AppError::Config(format!("{key} is required")))
}
