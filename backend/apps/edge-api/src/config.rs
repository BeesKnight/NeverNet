use std::env;

use crate::error::AppError;

#[derive(Debug)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub port: u16,
    pub identity_service_url: String,
    pub event_command_service_url: String,
    pub event_query_service_url: String,
    pub report_service_url: String,
    pub frontend_origins: Vec<String>,
    pub auth_cookie_secure: bool,
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        dotenvy::dotenv().ok();

        let database_url = required("DATABASE_URL")?;
        let jwt_secret = required("JWT_SECRET")?;
        let port = env::var("PORT")
            .ok()
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(8080);
        let identity_service_url = env::var("IDENTITY_SERVICE_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:50051".to_string());
        let event_command_service_url = env::var("EVENT_COMMAND_SERVICE_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:50052".to_string());
        let event_query_service_url = env::var("EVENT_QUERY_SERVICE_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:50053".to_string());
        let report_service_url =
            env::var("REPORT_SERVICE_URL").unwrap_or_else(|_| "http://127.0.0.1:50054".to_string());
        let frontend_origins = env::var("FRONTEND_ORIGINS")
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

        Ok(Self {
            database_url,
            jwt_secret,
            port,
            identity_service_url,
            event_command_service_url,
            event_query_service_url,
            report_service_url,
            frontend_origins,
            auth_cookie_secure,
        })
    }
}

fn required(key: &str) -> Result<String, AppError> {
    env::var(key).map_err(|_| AppError::Config(format!("{key} is required")))
}
