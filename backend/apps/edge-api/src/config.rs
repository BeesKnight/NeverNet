use std::{env, path::PathBuf};

use crate::error::AppError;

#[derive(Debug)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub port: u16,
    pub export_dir: PathBuf,
    pub identity_service_url: String,
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
        let export_dir = env::var("EXPORT_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("storage/exports"));
        let identity_service_url = env::var("IDENTITY_SERVICE_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:50051".to_string());
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
            export_dir,
            identity_service_url,
            frontend_origins,
            auth_cookie_secure,
        })
    }
}

fn required(key: &str) -> Result<String, AppError> {
    env::var(key).map_err(|_| AppError::Config(format!("{key} is required")))
}
