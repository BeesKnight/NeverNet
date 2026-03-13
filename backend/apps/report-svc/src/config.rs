use std::env;

use crate::error::AppError;

#[derive(Debug)]
pub struct Config {
    pub database_url: String,
    pub grpc_port: u16,
    pub metrics_port: u16,
    pub minio_endpoint: String,
    pub minio_bucket: String,
    pub minio_access_key: String,
    pub minio_secret_key: String,
    pub minio_region: String,
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        dotenvy::dotenv().ok();

        Ok(Self {
            database_url: required("DATABASE_URL")?,
            grpc_port: env::var("GRPC_PORT")
                .ok()
                .and_then(|value| value.parse::<u16>().ok())
                .unwrap_or(50054),
            metrics_port: env::var("METRICS_PORT")
                .ok()
                .and_then(|value| value.parse::<u16>().ok())
                .unwrap_or(9104),
            minio_endpoint: env::var("MINIO_ENDPOINT")
                .unwrap_or_else(|_| "http://127.0.0.1:9000".to_string()),
            minio_bucket: env::var("MINIO_BUCKET")
                .unwrap_or_else(|_| "eventdesign-exports".to_string()),
            minio_access_key: env::var("MINIO_ACCESS_KEY")
                .unwrap_or_else(|_| "eventdesign".to_string()),
            minio_secret_key: env::var("MINIO_SECRET_KEY")
                .unwrap_or_else(|_| "eventdesign123".to_string()),
            minio_region: env::var("MINIO_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
        })
    }
}

fn required(key: &str) -> Result<String, AppError> {
    env::var(key).map_err(|_| AppError::Config(format!("{key} is required")))
}
