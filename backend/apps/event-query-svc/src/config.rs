use std::env;

use crate::error::AppError;

#[derive(Debug)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub grpc_port: u16,
    pub metrics_port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        dotenvy::dotenv().ok();

        let database_url = required("DATABASE_URL")?;
        let redis_url =
            env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
        let grpc_port = env::var("GRPC_PORT")
            .ok()
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(50053);
        let metrics_port = env::var("METRICS_PORT")
            .ok()
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(9103);

        Ok(Self {
            database_url,
            redis_url,
            grpc_port,
            metrics_port,
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
    fn reads_default_ports_and_redis_url() {
        with_env(
            &[
                ("REDIS_URL", None),
                ("GRPC_PORT", None),
                ("METRICS_PORT", None),
            ],
            || {
                let config = Config::from_env().expect("config should be valid");
                assert_eq!(config.redis_url, "redis://127.0.0.1:6379");
                assert_eq!(config.grpc_port, 50053);
                assert_eq!(config.metrics_port, 9103);
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
