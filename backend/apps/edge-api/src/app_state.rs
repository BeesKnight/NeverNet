use std::sync::Arc;

use redis::Client;
use sqlx::PgPool;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub redis: Client,
    pub config: Arc<Config>,
}

impl AppState {
    pub fn new(pool: PgPool, redis: Client, config: Arc<Config>) -> Self {
        Self {
            pool,
            redis,
            config,
        }
    }
}
