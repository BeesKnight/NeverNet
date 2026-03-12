use std::{env, path::PathBuf};

use crate::error::AppError;

#[derive(Debug)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub port: u16,
    pub export_dir: PathBuf,
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

        Ok(Self {
            database_url,
            jwt_secret,
            port,
            export_dir,
        })
    }
}

fn required(key: &str) -> Result<String, AppError> {
    env::var(key).map_err(|_| AppError::Config(format!("{key} is required")))
}
