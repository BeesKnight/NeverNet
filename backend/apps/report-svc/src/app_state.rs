use std::sync::Arc;

use s3::Bucket;
use sqlx::PgPool;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub storage: Box<Bucket>,
}

impl AppState {
    pub fn new(pool: PgPool, storage: Box<Bucket>, _config: Arc<Config>) -> Self {
        Self { pool, storage }
    }
}
