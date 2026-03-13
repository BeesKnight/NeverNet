use std::env;

use http::Uri;

use crate::error::AppError;

#[derive(Debug)]
pub struct Config {
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
        let frontend_origins = parse_frontend_origins(
            &env::var("FRONTEND_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:3000,http://localhost:5173".to_string())
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>(),
        )?;
        let has_non_local_origin = frontend_origins
            .iter()
            .any(|origin| !is_local_origin(origin).unwrap_or(false));
        let auth_cookie_secure = env::var("AUTH_COOKIE_SECURE")
            .ok()
            .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "True"))
            .unwrap_or(has_non_local_origin);
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

        if has_non_local_origin && !auth_cookie_secure {
            return Err(AppError::Config(
                "AUTH_COOKIE_SECURE must be true when FRONTEND_ORIGINS contains non-local origins"
                    .to_string(),
            ));
        }

        Ok(Self {
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

fn parse_frontend_origins(origins: &[String]) -> Result<Vec<String>, AppError> {
    let mut validated = Vec::with_capacity(origins.len());

    for origin in origins {
        validate_frontend_origin(origin)?;
        validated.push(origin.clone());
    }

    Ok(validated)
}

fn validate_frontend_origin(origin: &str) -> Result<(), AppError> {
    let trimmed = origin.trim();
    if trimmed.is_empty() {
        return Err(AppError::Config(
            "FRONTEND_ORIGINS cannot contain empty values".to_string(),
        ));
    }

    if trimmed == "*" {
        return Err(AppError::Config(
            "FRONTEND_ORIGINS must not contain wildcard origins".to_string(),
        ));
    }

    let uri: Uri = trimmed
        .parse()
        .map_err(|_| AppError::Config(format!("Invalid frontend origin: {trimmed}")))?;
    let scheme = uri
        .scheme_str()
        .ok_or_else(|| AppError::Config(format!("Invalid frontend origin: {trimmed}")))?;
    let authority = uri
        .authority()
        .ok_or_else(|| AppError::Config(format!("Invalid frontend origin: {trimmed}")))?;
    let host = authority.host();

    if !matches!(scheme, "http" | "https") {
        return Err(AppError::Config(format!(
            "Frontend origin must use http or https: {trimmed}"
        )));
    }

    if !is_local_host(host) && scheme != "https" {
        return Err(AppError::Config(format!(
            "Non-local frontend origin must use https: {trimmed}"
        )));
    }

    if uri
        .path_and_query()
        .map(|value| value.as_str())
        .unwrap_or("/")
        != "/"
    {
        return Err(AppError::Config(format!(
            "Frontend origin must not contain a path: {trimmed}"
        )));
    }

    Ok(())
}

fn is_local_origin(origin: &str) -> Result<bool, AppError> {
    let uri: Uri = origin
        .parse()
        .map_err(|_| AppError::Config(format!("Invalid frontend origin: {origin}")))?;
    let authority = uri
        .authority()
        .ok_or_else(|| AppError::Config(format!("Invalid frontend origin: {origin}")))?;

    Ok(is_local_host(authority.host()))
}

fn is_local_host(host: &str) -> bool {
    matches!(host, "localhost" | "127.0.0.1" | "::1") || host.ends_with(".localhost")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    static ENV_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

    #[test]
    fn rejects_wildcard_origin() {
        with_env(
            &[
                ("FRONTEND_ORIGINS", Some("*")),
                ("AUTH_COOKIE_SECURE", None),
            ],
            || {
                let error = Config::from_env().expect_err("wildcard must be rejected");
                assert!(
                    error
                        .to_string()
                        .contains("must not contain wildcard origins")
                );
            },
        );
    }

    #[test]
    fn requires_secure_cookies_for_non_local_origins() {
        with_env(
            &[
                ("FRONTEND_ORIGINS", Some("https://app.eventdesign.local")),
                ("AUTH_COOKIE_SECURE", Some("false")),
            ],
            || {
                let error =
                    Config::from_env().expect_err("non-local origins require secure cookies");
                assert!(
                    error
                        .to_string()
                        .contains("AUTH_COOKIE_SECURE must be true")
                );
            },
        );
    }

    #[test]
    fn defaults_to_secure_cookies_for_non_local_https_origin() {
        with_env(
            &[
                ("FRONTEND_ORIGINS", Some("https://app.eventdesign.local")),
                ("AUTH_COOKIE_SECURE", None),
            ],
            || {
                let config = Config::from_env().expect("config should be valid");
                assert!(config.auth_cookie_secure);
            },
        );
    }

    fn with_env(vars: &[(&str, Option<&str>)], test: impl FnOnce()) {
        let _guard = ENV_MUTEX
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("env mutex poisoned");
        let keys: Vec<&str> = vars.iter().map(|(key, _)| *key).collect();
        let saved: Vec<(&str, Option<String>)> =
            keys.iter().map(|key| (*key, env::var(key).ok())).collect();

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
