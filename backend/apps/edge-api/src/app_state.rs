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
