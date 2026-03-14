use std::sync::Arc;

use redis::Client;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub redis: Client,
    pub config: Arc<Config>,
}

impl AppState {
    pub fn new(redis: Client, config: Arc<Config>) -> Self {
        Self { redis, config }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores_config_and_redis_client() {
        let redis = Client::open("redis://127.0.0.1:6379").expect("redis url should be valid");
        let config = Arc::new(Config {
            redis_url: "redis://127.0.0.1:6379".to_string(),
            port: 8080,
            metrics_port: 9100,
            identity_service_url: "http://127.0.0.1:50051".to_string(),
            event_command_service_url: "http://127.0.0.1:50052".to_string(),
            event_query_service_url: "http://127.0.0.1:50053".to_string(),
            report_service_url: "http://127.0.0.1:50054".to_string(),
            frontend_origins: vec!["http://localhost:3000".to_string()],
            auth_cookie_secure: false,
            rate_limit_window_seconds: 60,
            rate_limit_requests_per_window: 300,
            auth_rate_limit_requests_per_window: 20,
        });

        let state = AppState::new(redis, config.clone());

        assert!(Arc::ptr_eq(&state.config, &config));
        assert_eq!(state.config.port, 8080);
        assert_eq!(state.config.redis_url, "redis://127.0.0.1:6379");
    }
}
