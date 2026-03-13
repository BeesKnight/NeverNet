use std::time::Duration;

use async_nats::jetstream::{
    self,
    consumer::{AckPolicy, pull},
};
use messaging::{
    DOMAIN_EVENTS_STREAM, EXPORT_CONSUMER, MessagingConfig, PROJECTION_CONSUMER, subjects,
};
use s3::{Bucket, Region, creds::Credentials};

#[derive(Debug, thiserror::Error)]
enum BootstrapError {
    #[error("{0}")]
    Config(String),
    #[error("{0}")]
    Internal(String),
}

struct Config {
    messaging: MessagingConfig,
    minio_endpoint: String,
    minio_bucket: String,
    minio_access_key: String,
    minio_secret_key: String,
    minio_region: String,
}

impl Config {
    fn from_env() -> Self {
        dotenvy::dotenv().ok();

        Self {
            messaging: MessagingConfig::from_env(),
            minio_endpoint: std::env::var("MINIO_ENDPOINT")
                .unwrap_or_else(|_| "http://127.0.0.1:9000".to_string()),
            minio_bucket: std::env::var("MINIO_BUCKET")
                .unwrap_or_else(|_| "eventdesign-exports".to_string()),
            minio_access_key: std::env::var("MINIO_ACCESS_KEY")
                .unwrap_or_else(|_| "eventdesign".to_string()),
            minio_secret_key: std::env::var("MINIO_SECRET_KEY")
                .unwrap_or_else(|_| "eventdesign123".to_string()),
            minio_region: std::env::var("MINIO_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    observability::init_tracing("infra-bootstrap", "infra_bootstrap=info");

    let config = Config::from_env();
    let nats = connect_nats(&config.messaging.nats_url).await?;
    let jetstream = async_nats::jetstream::new(nats);
    ensure_domain_stream(&jetstream).await?;
    ensure_projection_consumer(&jetstream).await?;
    ensure_export_consumer(&jetstream).await?;

    let bucket = build_storage_bucket(&config)?;
    ensure_bucket(&bucket, &config.minio_bucket).await?;

    tracing::info!("infrastructure bootstrap completed successfully");
    Ok(())
}

async fn connect_nats(nats_url: &str) -> Result<async_nats::Client, BootstrapError> {
    let mut last_error = None;

    for attempt in 1..=10 {
        match async_nats::connect(nats_url).await {
            Ok(client) => return Ok(client),
            Err(error) => {
                tracing::warn!(
                    "could not connect to NATS on attempt {} of 10: {}",
                    attempt,
                    error
                );
                last_error = Some(error);
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }

    match last_error {
        Some(error) => Err(BootstrapError::Internal(format!(
            "Could not connect to NATS after retries: {error}"
        ))),
        None => Err(BootstrapError::Internal(
            "NATS connection attempts exhausted".to_string(),
        )),
    }
}

async fn ensure_domain_stream(context: &jetstream::Context) -> Result<(), BootstrapError> {
    context
        .get_or_create_stream(jetstream::stream::Config {
            name: DOMAIN_EVENTS_STREAM.to_string(),
            subjects: vec![subjects::ALL.to_string()],
            max_messages: 50_000,
            ..Default::default()
        })
        .await
        .map_err(|error| {
            BootstrapError::Internal(format!("Could not create domain stream: {error}"))
        })?;

    Ok(())
}

async fn ensure_projection_consumer(context: &jetstream::Context) -> Result<(), BootstrapError> {
    let stream = context
        .get_stream(DOMAIN_EVENTS_STREAM)
        .await
        .map_err(|error| {
            BootstrapError::Internal(format!("Could not get domain stream: {error}"))
        })?;

    stream
        .get_or_create_consumer(
            PROJECTION_CONSUMER,
            pull::Config {
                durable_name: Some(PROJECTION_CONSUMER.to_string()),
                ack_policy: AckPolicy::Explicit,
                ack_wait: Duration::from_secs(60),
                filter_subject: subjects::ALL.to_string(),
                ..Default::default()
            },
        )
        .await
        .map_err(|error| {
            BootstrapError::Internal(format!("Could not create projection consumer: {error}"))
        })?;

    Ok(())
}

async fn ensure_export_consumer(context: &jetstream::Context) -> Result<(), BootstrapError> {
    let stream = context
        .get_stream(DOMAIN_EVENTS_STREAM)
        .await
        .map_err(|error| {
            BootstrapError::Internal(format!("Could not get domain stream: {error}"))
        })?;

    stream
        .get_or_create_consumer(
            EXPORT_CONSUMER,
            pull::Config {
                durable_name: Some(EXPORT_CONSUMER.to_string()),
                ack_policy: AckPolicy::Explicit,
                ack_wait: Duration::from_secs(600),
                filter_subject: subjects::EXPORT_REQUESTED.to_string(),
                ..Default::default()
            },
        )
        .await
        .map_err(|error| {
            BootstrapError::Internal(format!("Could not create export consumer: {error}"))
        })?;

    Ok(())
}

fn build_storage_bucket(config: &Config) -> Result<Box<Bucket>, BootstrapError> {
    let region = Region::Custom {
        region: config.minio_region.clone(),
        endpoint: config.minio_endpoint.clone(),
    };
    let credentials = Credentials::new(
        Some(&config.minio_access_key),
        Some(&config.minio_secret_key),
        None,
        None,
        None,
    )
    .map_err(|error| {
        BootstrapError::Config(format!("Could not build MinIO credentials: {error}"))
    })?;

    Bucket::new(&config.minio_bucket, region, credentials)
        .map(|bucket| bucket.with_path_style())
        .map_err(|error| {
            BootstrapError::Config(format!("Could not build MinIO bucket client: {error}"))
        })
}

async fn ensure_bucket(bucket: &Bucket, bucket_name: &str) -> Result<(), BootstrapError> {
    for attempt in 1..=10 {
        match bucket.exists().await {
            Ok(true) => return Ok(()),
            Ok(false) => {
                unsafe {
                    std::env::set_var("RUST_S3_SKIP_LOCATION_CONSTRAINT", "1");
                }
                let credentials = bucket.credentials().await.map_err(|error| {
                    BootstrapError::Internal(format!(
                        "Could not read MinIO credentials for {bucket_name}: {error}"
                    ))
                })?;
                let region = bucket.region();

                match Bucket::create_with_path_style(
                    bucket_name,
                    region,
                    credentials,
                    s3::bucket_ops::BucketConfiguration::default(),
                )
                .await
                {
                    Ok(_) => return Ok(()),
                    Err(error) => {
                        tracing::warn!(
                            "could not create MinIO bucket {} on attempt {} of 10: {}",
                            bucket_name,
                            attempt,
                            error
                        );
                    }
                }
            }
            Err(error) => {
                tracing::warn!(
                    "could not check MinIO bucket {} on attempt {} of 10: {}",
                    bucket_name,
                    attempt,
                    error
                );
            }
        }

        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    Err(BootstrapError::Internal(format!(
        "MinIO bucket {bucket_name} is not reachable after retries"
    )))
}
