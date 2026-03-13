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
