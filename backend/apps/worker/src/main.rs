use std::time::Duration;

use cache::CacheConfig;
use messaging::MessagingConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    observability::init_tracing("worker=info");

    let cache = CacheConfig::from_env();
    let messaging = MessagingConfig::from_env();
    let minio_endpoint =
        std::env::var("MINIO_ENDPOINT").unwrap_or_else(|_| "http://localhost:9000".to_string());

    tracing::info!(
        redis_url = %cache.redis_url,
        nats_url = %messaging.nats_url,
        minio_endpoint = %minio_endpoint,
        "worker skeleton started; background processors will migrate in Phase 2"
    );

    loop {
        tokio::time::sleep(Duration::from_secs(300)).await;
    }
}
