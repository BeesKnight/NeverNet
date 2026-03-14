pub const DEFAULT_REDIS_URL: &str = "redis://localhost:6379";

#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub redis_url: String,
}

impl CacheConfig {
    pub fn from_env() -> Self {
        Self {
            redis_url: std::env::var("REDIS_URL").unwrap_or_else(|_| DEFAULT_REDIS_URL.to_string()),
        }
    }
}

pub fn dashboard_key(user_id: &str) -> String {
    format!("dashboard:{user_id}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_default_cache_config() {
        unsafe {
            std::env::remove_var("REDIS_URL");
        }

        let config = CacheConfig::from_env();

        assert_eq!(config.redis_url, DEFAULT_REDIS_URL);
    }

    #[test]
    fn builds_dashboard_cache_key() {
        assert_eq!(dashboard_key("user-1"), "dashboard:user-1");
    }
}
