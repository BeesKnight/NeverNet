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

#[cfg(test)]
mod tests {
    use std::sync::{Mutex, OnceLock};

    use super::*;

    static ENV_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

    #[test]
    fn reads_default_storage_configuration() {
        with_env(
            &[
                ("GRPC_PORT", None),
                ("METRICS_PORT", None),
                ("MINIO_ENDPOINT", None),
                ("MINIO_BUCKET", None),
                ("MINIO_ACCESS_KEY", None),
                ("MINIO_SECRET_KEY", None),
                ("MINIO_REGION", None),
            ],
            || {
                let config = Config::from_env().expect("config should be valid");
                assert_eq!(config.grpc_port, 50054);
                assert_eq!(config.metrics_port, 9104);
                assert_eq!(config.minio_bucket, "eventdesign-exports");
                assert_eq!(config.minio_region, "us-east-1");
            },
        );
    }

    fn with_env(vars: &[(&str, Option<&str>)], test: impl FnOnce()) {
        let _guard = ENV_MUTEX
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("env mutex poisoned");
        let saved: Vec<(&str, Option<String>)> = vars
            .iter()
            .map(|(key, _)| (*key, env::var(key).ok()))
            .collect();

        for (key, value) in vars {
            match value {
                Some(value) => unsafe { env::set_var(key, value) },
                None => unsafe { env::remove_var(key) },
            }
        }

        test();

        for (key, value) in saved {
            match value {
                Some(value) => unsafe { env::set_var(key, value) },
                None => unsafe { env::remove_var(key) },
            }
        }
    }
}
