use std::time::{Duration, Instant};

use async_nats::jetstream::{
    self,
    consumer::{AckPolicy, pull},
    context::Publish,
};
use cache::{CacheConfig, dashboard_key};
use chrono::{DateTime, NaiveDate, Utc};
use futures::StreamExt;
use messaging::{
    DOMAIN_EVENTS_STREAM, DomainEventEnvelope, EXPORT_CONSUMER, MessagingConfig,
    PROJECTION_CONSUMER, subject_for_event_type, subjects,
};
use persistence::connect_pool;
use printpdf::{BuiltinFont, Mm, PdfDocument};
use redis::{AsyncCommands, Script};
use rust_xlsxwriter::Workbook;
use s3::{Bucket, Region, creds::Credentials};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{FromRow, PgPool, Postgres, QueryBuilder, Transaction};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    pool: PgPool,
    redis: redis::Client,
    jetstream: jetstream::Context,
    bucket: Box<Bucket>,
}

#[derive(Debug, thiserror::Error)]
enum WorkerError {
    #[error("{0}")]
    Config(String),
    #[error("{0}")]
    Internal(String),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Redis(#[from] redis::RedisError),
    #[error(transparent)]
    Nats(#[from] async_nats::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
}

#[derive(Debug)]
struct Config {
    database_url: String,
    cache: CacheConfig,
    messaging: MessagingConfig,
    metrics_port: u16,
    minio_endpoint: String,
    minio_bucket: String,
    minio_access_key: String,
    minio_secret_key: String,
    minio_region: String,
}

impl Config {
    fn from_env() -> Result<Self, WorkerError> {
        dotenvy::dotenv().ok();

        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .map_err(|_| WorkerError::Config("DATABASE_URL is required".to_string()))?,
            cache: CacheConfig::from_env(),
            messaging: MessagingConfig::from_env(),
            metrics_port: std::env::var("METRICS_PORT")
                .ok()
                .and_then(|value| value.parse::<u16>().ok())
                .unwrap_or(9105),
            minio_endpoint: std::env::var("MINIO_ENDPOINT")
                .unwrap_or_else(|_| "http://127.0.0.1:9000".to_string()),
            minio_bucket: std::env::var("MINIO_BUCKET")
                .unwrap_or_else(|_| "eventdesign-exports".to_string()),
            minio_access_key: std::env::var("MINIO_ACCESS_KEY")
                .unwrap_or_else(|_| "eventdesign".to_string()),
            minio_secret_key: std::env::var("MINIO_SECRET_KEY")
                .unwrap_or_else(|_| "eventdesign123".to_string()),
            minio_region: std::env::var("MINIO_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
        })
    }
}

#[derive(Debug, Clone, FromRow)]
struct OutboxEventRow {
    id: Uuid,
    aggregate_type: String,
    aggregate_id: Uuid,
    event_type: String,
    event_version: i32,
    payload_json: Value,
    occurred_at: DateTime<Utc>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
struct CategoryPayload {
    user_id: Uuid,
    category_id: Uuid,
    name: String,
    color: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
struct EventPayload {
    user_id: Uuid,
    event_id: Uuid,
    category_id: Uuid,
    title: String,
    description: String,
    location: String,
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
    budget: f64,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
struct EventStatusChangedPayload {
    user_id: Uuid,
    event_id: Uuid,
    title: String,
    previous_status: String,
    new_status: String,
    occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExportPayload {
    export_id: Uuid,
    user_id: Uuid,
    report_type: String,
    format: String,
    status: String,
    filters: Value,
    object_key: Option<String>,
    error_message: Option<String>,
    created_at: DateTime<Utc>,
    started_at: Option<DateTime<Utc>>,
    finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow)]
struct ProjectionEventRow {
    event_id: Uuid,
    user_id: Uuid,
    category_id: Uuid,
    category_name: String,
    category_color: String,
    title: String,
    description: String,
    location: String,
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
    budget: f64,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, FromRow)]
struct ExportJobRow {
    id: Uuid,
    user_id: Uuid,
    report_type: String,
    format: String,
    status: String,
    filters: Value,
    object_key: Option<String>,
    content_type: Option<String>,
    error_message: Option<String>,
    created_at: DateTime<Utc>,
    started_at: Option<DateTime<Utc>>,
    updated_at: DateTime<Utc>,
    finished_at: Option<DateTime<Utc>>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, FromRow)]
struct ReportEventRow {
    id: Uuid,
    user_id: Uuid,
    category_id: Uuid,
    category_name: String,
    category_color: String,
    title: String,
    description: String,
    location: String,
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
    budget: f64,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct ExportFilters {
    status: Option<String>,
    category_id: Option<Uuid>,
    start_date: Option<NaiveDate>,
    end_date: Option<NaiveDate>,
}

#[derive(Debug, Clone)]
struct ReportSummary {
    period_start: Option<NaiveDate>,
    period_end: Option<NaiveDate>,
    total_events: usize,
    total_budget: f64,
    events: Vec<ReportEventRow>,
}

#[derive(Debug, Clone)]
struct ExportArtifact {
    object_key: String,
    content_type: &'static str,
    bytes: Vec<u8>,
}

struct ExportLock {
    key: String,
    token: String,
}

enum MessageDisposition {
    Ack,
    SkipAck,
}

#[allow(clippy::large_enum_variant)]
enum ExportClaim {
    Missing,
    AlreadyProcessed,
    InProgress,
    Claimed(ExportJobRow),
}

const RELAY_BATCH_SIZE: usize = 50;
const EXPORT_PROCESSING_TIMEOUT_MINUTES: i64 = 15;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    observability::init_tracing("worker", "worker=info");

    let config = Config::from_env()?;
    observability::spawn_metrics_server("worker", config.metrics_port);
    let pool = connect_pool(&config.database_url, 10).await?;
    let redis = redis::Client::open(config.cache.redis_url.clone())?;
    let nats = async_nats::connect(config.messaging.nats_url.clone()).await?;
    let jetstream = async_nats::jetstream::new(nats);
    ensure_domain_stream(&jetstream).await?;
    let bucket = build_storage_bucket(&config)?;
    ensure_bucket(&bucket, &config.minio_bucket).await;

    let state = AppState {
        pool,
        redis,
        jetstream,
        bucket,
    };

    tracing::info!("worker started with outbox relay, projections, and export processor");

    tokio::try_join!(
        run_outbox_relay(state.clone()),
        run_projection_consumer(state.clone()),
        run_export_consumer(state),
    )?;

    Ok(())
}

async fn run_outbox_relay(state: AppState) -> Result<(), WorkerError> {
    loop {
        if let Err(error) = relay_outbox_batch(&state).await {
            tracing::error!("outbox relay error: {}", error);
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

async fn run_projection_consumer(state: AppState) -> Result<(), WorkerError> {
    let stream = state
        .jetstream
        .get_stream(DOMAIN_EVENTS_STREAM)
        .await
        .map_err(|error| WorkerError::Internal(format!("Could not get domain stream: {error}")))?;
    let consumer = stream
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
            WorkerError::Internal(format!("Could not create projection consumer: {error}"))
        })?;

    let mut messages = consumer.messages().await.map_err(|error| {
        WorkerError::Internal(format!("Could not subscribe projection consumer: {error}"))
    })?;

    while let Some(next) = messages.next().await {
        match next {
            Ok(message) => match process_projection_message(&state, &message).await {
                Ok(MessageDisposition::Ack) => {
                    if let Err(error) = message.ack().await {
                        tracing::error!("could not ack projection message: {}", error);
                    }
                }
                Ok(MessageDisposition::SkipAck) => {}
                Err(error) => tracing::error!("projection message failed: {}", error),
            },
            Err(error) => {
                tracing::error!("projection consumer stream error: {}", error);
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }

    Ok(())
}

async fn run_export_consumer(state: AppState) -> Result<(), WorkerError> {
    let stream = state
        .jetstream
        .get_stream(DOMAIN_EVENTS_STREAM)
        .await
        .map_err(|error| WorkerError::Internal(format!("Could not get domain stream: {error}")))?;
    let consumer = stream
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
            WorkerError::Internal(format!("Could not create export consumer: {error}"))
        })?;

    let mut messages = consumer.messages().await.map_err(|error| {
        WorkerError::Internal(format!("Could not subscribe export consumer: {error}"))
    })?;

    while let Some(next) = messages.next().await {
        match next {
            Ok(message) => match process_export_message(&state, &message).await {
                Ok(MessageDisposition::Ack) => {
                    if let Err(error) = message.ack().await {
                        tracing::error!("could not ack export message: {}", error);
                    }
                }
                Ok(MessageDisposition::SkipAck) => {}
                Err(error) => tracing::error!("export message failed: {}", error),
            },
            Err(error) => {
                tracing::error!("export consumer stream error: {}", error);
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }

    Ok(())
}

async fn relay_outbox_batch(state: &AppState) -> Result<(), WorkerError> {
    for _ in 0..RELAY_BATCH_SIZE {
        if !relay_next_outbox_row(state).await? {
            break;
        }
    }

    Ok(())
}

async fn relay_next_outbox_row(state: &AppState) -> Result<bool, WorkerError> {
    let mut tx = state.pool.begin().await?;
    let row = sqlx::query_as::<_, OutboxEventRow>(
        r#"
        SELECT id, aggregate_type, aggregate_id, event_type, event_version, payload_json, occurred_at
        FROM outbox_events
        WHERE published_at IS NULL
        ORDER BY occurred_at ASC
        LIMIT 1
        FOR UPDATE SKIP LOCKED
        "#,
    )
    .fetch_optional(&mut *tx)
    .await?;

    let Some(row) = row else {
        tx.commit().await?;
        return Ok(false);
    };

    let publish_result = publish_outbox_event(state, &row).await;

    match publish_result {
        Ok(()) => mark_outbox_published(&mut tx, row.id).await?,
        Err(error) => {
            tracing::warn!(
                outbox_event_id = %row.id,
                event_type = %row.event_type,
                "could not publish outbox event: {}",
                error
            );
            record_outbox_publish_failure(&mut tx, row.id, &error.to_string()).await?;
        }
    }

    tx.commit().await?;
    Ok(true)
}

async fn publish_outbox_event(state: &AppState, row: &OutboxEventRow) -> Result<(), WorkerError> {
    let envelope = DomainEventEnvelope {
        id: row.id.to_string(),
        aggregate_type: row.aggregate_type.clone(),
        aggregate_id: row.aggregate_id.to_string(),
        event_type: row.event_type.clone(),
        event_version: row.event_version,
        occurred_at: row.occurred_at,
        payload: row.payload_json.clone(),
    };
    let payload = serde_json::to_vec(&envelope)?;
    let publish = Publish::build()
        .payload(payload.into())
        .message_id(row.id.to_string());
    let ack = state
        .jetstream
        .send_publish(subject_for_event_type(&row.event_type), publish)
        .await
        .map_err(|error| WorkerError::Internal(format!("JetStream publish failed: {error}")))?;

    ack.await
        .map_err(|error| WorkerError::Internal(format!("JetStream ack failed: {error}")))?;

    Ok(())
}

async fn mark_outbox_published(
    tx: &mut Transaction<'_, Postgres>,
    outbox_event_id: Uuid,
) -> Result<(), WorkerError> {
    sqlx::query(
        r#"
        UPDATE outbox_events
        SET
            published_at = COALESCE(published_at, NOW()),
            publish_attempts = publish_attempts + 1,
            last_error = NULL
        WHERE id = $1
        "#,
    )
    .bind(outbox_event_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn record_outbox_publish_failure(
    tx: &mut Transaction<'_, Postgres>,
    outbox_event_id: Uuid,
    error_message: &str,
) -> Result<(), WorkerError> {
    sqlx::query(
        r#"
        UPDATE outbox_events
        SET publish_attempts = publish_attempts + 1, last_error = $2
        WHERE id = $1
        "#,
    )
    .bind(outbox_event_id)
    .bind(error_message)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn process_projection_message(
    state: &AppState,
    message: &async_nats::jetstream::Message,
) -> Result<MessageDisposition, WorkerError> {
    let envelope = match serde_json::from_slice::<DomainEventEnvelope>(&message.payload) {
        Ok(envelope) => envelope,
        Err(error) => {
            tracing::error!("invalid projection envelope: {}", error);
            return Ok(MessageDisposition::Ack);
        }
    };
    let message_id = match Uuid::parse_str(&envelope.id) {
        Ok(id) => id,
        Err(error) => {
            tracing::error!("invalid envelope id {}: {}", envelope.id, error);
            return Ok(MessageDisposition::Ack);
        }
    };
    let lag_seconds = (Utc::now() - envelope.occurred_at)
        .num_milliseconds()
        .max(0) as f64
        / 1_000.0;
    observability::set_projection_lag("read_models", lag_seconds);
    observability::set_queue_lag("projection_consumer", lag_seconds);

    let mut tx = state.pool.begin().await?;
    if !claim_processed_message(&mut tx, PROJECTION_CONSUMER, message_id).await? {
        tx.commit().await?;
        return Ok(MessageDisposition::Ack);
    }

    let invalidate_user = match envelope.event_type.as_str() {
        "category.created" => {
            let payload: CategoryPayload = serde_json::from_value(envelope.payload)?;
            insert_activity(
                &mut tx,
                message_id,
                payload.user_id,
                "category",
                payload.category_id,
                "created",
                &payload.name,
                payload.updated_at,
            )
            .await?;
            Some(payload.user_id)
        }
        "category.updated" => {
            let payload: CategoryPayload = serde_json::from_value(envelope.payload)?;
            update_category_projection_rows(&mut tx, &payload).await?;
            insert_activity(
                &mut tx,
                message_id,
                payload.user_id,
                "category",
                payload.category_id,
                "updated",
                &payload.name,
                payload.updated_at,
            )
            .await?;
            Some(payload.user_id)
        }
        "category.deleted" => {
            let payload: CategoryPayload = serde_json::from_value(envelope.payload)?;
            insert_activity(
                &mut tx,
                message_id,
                payload.user_id,
                "category",
                payload.category_id,
                "deleted",
                &payload.name,
                payload.updated_at,
            )
            .await?;
            Some(payload.user_id)
        }
        "event.created" => {
            let payload: EventPayload = serde_json::from_value(envelope.payload)?;
            upsert_event_projection_rows(&mut tx, payload.event_id).await?;
            refresh_dashboard_projection(&mut tx, payload.user_id).await?;
            insert_activity(
                &mut tx,
                message_id,
                payload.user_id,
                "event",
                payload.event_id,
                "created",
                &payload.title,
                payload.updated_at,
            )
            .await?;
            Some(payload.user_id)
        }
        "event.updated" => {
            let payload: EventPayload = serde_json::from_value(envelope.payload)?;
            upsert_event_projection_rows(&mut tx, payload.event_id).await?;
            refresh_dashboard_projection(&mut tx, payload.user_id).await?;
            insert_activity(
                &mut tx,
                message_id,
                payload.user_id,
                "event",
                payload.event_id,
                "updated",
                &payload.title,
                payload.updated_at,
            )
            .await?;
            Some(payload.user_id)
        }
        "event.deleted" => {
            let payload: EventPayload = serde_json::from_value(envelope.payload)?;
            delete_event_projection_rows(&mut tx, payload.event_id).await?;
            refresh_dashboard_projection(&mut tx, payload.user_id).await?;
            insert_activity(
                &mut tx,
                message_id,
                payload.user_id,
                "event",
                payload.event_id,
                "deleted",
                &payload.title,
                payload.updated_at,
            )
            .await?;
            Some(payload.user_id)
        }
        "event.status_changed" => {
            let payload: EventStatusChangedPayload = serde_json::from_value(envelope.payload)?;
            refresh_dashboard_projection(&mut tx, payload.user_id).await?;
            insert_activity(
                &mut tx,
                message_id,
                payload.user_id,
                "event",
                payload.event_id,
                "status_changed",
                &format!(
                    "{}: {} -> {}",
                    payload.title, payload.previous_status, payload.new_status
                ),
                payload.occurred_at,
            )
            .await?;
            Some(payload.user_id)
        }
        _ => None,
    };

    tx.commit().await?;

    if let Some(user_id) = invalidate_user {
        invalidate_dashboard_cache(&state.redis, user_id).await;
    }

    Ok(MessageDisposition::Ack)
}

async fn process_export_message(
    state: &AppState,
    message: &async_nats::jetstream::Message,
) -> Result<MessageDisposition, WorkerError> {
    let envelope = match serde_json::from_slice::<DomainEventEnvelope>(&message.payload) {
        Ok(envelope) => envelope,
        Err(error) => {
            tracing::error!("invalid export envelope: {}", error);
            return Ok(MessageDisposition::Ack);
        }
    };
    let message_id = match Uuid::parse_str(&envelope.id) {
        Ok(id) => id,
        Err(error) => {
            tracing::error!("invalid export envelope id {}: {}", envelope.id, error);
            return Ok(MessageDisposition::Ack);
        }
    };
    let payload: ExportPayload = match serde_json::from_value(envelope.payload) {
        Ok(payload) => payload,
        Err(error) => {
            tracing::error!("invalid export payload: {}", error);
            return Ok(MessageDisposition::Ack);
        }
    };
    let queue_lag_seconds = (Utc::now() - envelope.occurred_at)
        .num_milliseconds()
        .max(0) as f64
        / 1_000.0;
    observability::set_queue_lag("export_consumer", queue_lag_seconds);

    let export_lock = match acquire_export_lock(&state.redis, payload.export_id).await? {
        Some(lock) => lock,
        None => return Ok(MessageDisposition::SkipAck),
    };
    let started_at = Instant::now();

    let result: Result<MessageDisposition, WorkerError> = async {
        if processed_message_exists(&state.pool, EXPORT_CONSUMER, message_id).await? {
            return Ok(MessageDisposition::Ack);
        }

        let job = match claim_export_job(&state.pool, payload.export_id, message_id).await? {
            ExportClaim::Missing | ExportClaim::AlreadyProcessed => {
                return Ok(MessageDisposition::Ack);
            }
            ExportClaim::InProgress => {
                return Ok(MessageDisposition::SkipAck);
            }
            ExportClaim::Claimed(job) => job,
        };

        let filters: ExportFilters = serde_json::from_value(job.filters.clone())?;
        let summary = generate_report_summary(&state.pool, job.user_id, &filters).await?;
        let artifact = build_export_artifact(&job, &summary)?;
        upload_export_artifact(&state.bucket, &artifact).await?;
        mark_export_completed(&state.pool, &job, &artifact, message_id).await?;

        Ok(MessageDisposition::Ack)
    }
    .await;

    release_export_lock(&state.redis, export_lock).await;

    match result {
        Ok(disposition) => {
            observability::observe_export_duration(
                &payload.format,
                "completed",
                started_at.elapsed(),
            );
            Ok(disposition)
        }
        Err(error) => {
            tracing::error!("export {} failed: {}", payload.export_id, error);
            observability::observe_export_duration(&payload.format, "failed", started_at.elapsed());
            if let Err(mark_error) = mark_export_failed(
                &state.pool,
                payload.export_id,
                &error.to_string(),
                message_id,
            )
            .await
            {
                tracing::error!(
                    "could not persist export failure for {}: {}",
                    payload.export_id,
                    mark_error
                );
                return Err(mark_error);
            }
            Ok(MessageDisposition::Ack)
        }
    }
}

async fn ensure_domain_stream(context: &jetstream::Context) -> Result<(), WorkerError> {
    context
        .get_or_create_stream(jetstream::stream::Config {
            name: DOMAIN_EVENTS_STREAM.to_string(),
            subjects: vec![subjects::ALL.to_string()],
            max_messages: 50_000,
            ..Default::default()
        })
        .await
        .map_err(|error| {
            WorkerError::Internal(format!("Could not create domain stream: {error}"))
        })?;

    Ok(())
}

fn build_storage_bucket(config: &Config) -> Result<Box<Bucket>, WorkerError> {
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
        WorkerError::Internal(format!("Could not build MinIO credentials: {error}"))
    })?;
    Bucket::new(&config.minio_bucket, region, credentials)
        .map(|bucket| bucket.with_path_style())
        .map_err(|error| {
            WorkerError::Internal(format!("Could not build MinIO bucket client: {error}"))
        })
}

async fn ensure_bucket(bucket: &Bucket, bucket_name: &str) {
    if bucket.exists().await.unwrap_or(false) {
        return;
    }

    unsafe {
        std::env::set_var("RUST_S3_SKIP_LOCATION_CONSTRAINT", "1");
    }
    let credentials = match bucket.credentials().await {
        Ok(credentials) => credentials,
        Err(error) => {
            tracing::warn!("could not read MinIO credentials for {bucket_name}: {error}");
            return;
        }
    };
    let region = bucket.region();
    if let Err(error) = Bucket::create_with_path_style(
        bucket_name,
        region,
        credentials,
        s3::bucket_ops::BucketConfiguration::default(),
    )
    .await
    {
        tracing::warn!("could not ensure MinIO bucket {bucket_name}: {error}");
    }
}

async fn claim_processed_message(
    tx: &mut Transaction<'_, Postgres>,
    consumer_name: &str,
    message_id: Uuid,
) -> Result<bool, WorkerError> {
    Ok(insert_processed_message(tx, consumer_name, message_id).await? == 1)
}

async fn processed_message_exists(
    pool: &PgPool,
    consumer_name: &str,
    message_id: Uuid,
) -> Result<bool, WorkerError> {
    let exists = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM processed_messages
            WHERE consumer_name = $1 AND message_id = $2
        )
        "#,
    )
    .bind(consumer_name)
    .bind(message_id)
    .fetch_one(pool)
    .await?;

    Ok(exists)
}

async fn insert_processed_message(
    tx: &mut Transaction<'_, Postgres>,
    consumer_name: &str,
    message_id: Uuid,
) -> Result<u64, WorkerError> {
    let result = sqlx::query(
        r#"
        INSERT INTO processed_messages (consumer_name, message_id)
        VALUES ($1, $2)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(consumer_name)
    .bind(message_id)
    .execute(&mut **tx)
    .await?;

    Ok(result.rows_affected())
}

async fn fetch_projection_source(
    tx: &mut Transaction<'_, Postgres>,
    event_id: Uuid,
) -> Result<Option<ProjectionEventRow>, WorkerError> {
    let row = sqlx::query_as::<_, ProjectionEventRow>(
        r#"
        SELECT
            e.id AS event_id,
            e.user_id,
            e.category_id,
            c.name AS category_name,
            c.color AS category_color,
            e.title,
            e.description,
            e.location,
            e.starts_at,
            e.ends_at,
            e.budget,
            e.status,
            e.created_at,
            e.updated_at
        FROM events e
        INNER JOIN categories c ON c.id = e.category_id
        WHERE e.id = $1
        "#,
    )
    .bind(event_id)
    .fetch_optional(&mut **tx)
    .await?;

    Ok(row)
}

async fn upsert_event_projection_rows(
    tx: &mut Transaction<'_, Postgres>,
    event_id: Uuid,
) -> Result<(), WorkerError> {
    if let Some(row) = fetch_projection_source(tx, event_id).await? {
        sqlx::query(
            r#"
            INSERT INTO event_list_projection (
                event_id,
                user_id,
                category_id,
                category_name,
                category_color,
                title,
                description,
                location,
                starts_at,
                ends_at,
                budget,
                status,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            ON CONFLICT (event_id) DO UPDATE
            SET
                user_id = EXCLUDED.user_id,
                category_id = EXCLUDED.category_id,
                category_name = EXCLUDED.category_name,
                category_color = EXCLUDED.category_color,
                title = EXCLUDED.title,
                description = EXCLUDED.description,
                location = EXCLUDED.location,
                starts_at = EXCLUDED.starts_at,
                ends_at = EXCLUDED.ends_at,
                budget = EXCLUDED.budget,
                status = EXCLUDED.status,
                created_at = EXCLUDED.created_at,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(row.event_id)
        .bind(row.user_id)
        .bind(row.category_id)
        .bind(&row.category_name)
        .bind(&row.category_color)
        .bind(&row.title)
        .bind(&row.description)
        .bind(&row.location)
        .bind(row.starts_at)
        .bind(row.ends_at)
        .bind(row.budget)
        .bind(&row.status)
        .bind(row.created_at)
        .bind(row.updated_at)
        .execute(&mut **tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO calendar_projection (
                event_id,
                user_id,
                date_bucket,
                title,
                starts_at,
                ends_at,
                status,
                category_color,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (event_id) DO UPDATE
            SET
                user_id = EXCLUDED.user_id,
                date_bucket = EXCLUDED.date_bucket,
                title = EXCLUDED.title,
                starts_at = EXCLUDED.starts_at,
                ends_at = EXCLUDED.ends_at,
                status = EXCLUDED.status,
                category_color = EXCLUDED.category_color,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(row.event_id)
        .bind(row.user_id)
        .bind(row.starts_at.date_naive())
        .bind(&row.title)
        .bind(row.starts_at)
        .bind(row.ends_at)
        .bind(&row.status)
        .bind(&row.category_color)
        .bind(row.updated_at)
        .execute(&mut **tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO report_projection (
                event_id,
                user_id,
                category_id,
                category_name,
                category_color,
                title,
                description,
                location,
                starts_at,
                ends_at,
                budget,
                status,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            ON CONFLICT (event_id) DO UPDATE
            SET
                user_id = EXCLUDED.user_id,
                category_id = EXCLUDED.category_id,
                category_name = EXCLUDED.category_name,
                category_color = EXCLUDED.category_color,
                title = EXCLUDED.title,
                description = EXCLUDED.description,
                location = EXCLUDED.location,
                starts_at = EXCLUDED.starts_at,
                ends_at = EXCLUDED.ends_at,
                budget = EXCLUDED.budget,
                status = EXCLUDED.status,
                created_at = EXCLUDED.created_at,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(row.event_id)
        .bind(row.user_id)
        .bind(row.category_id)
        .bind(&row.category_name)
        .bind(&row.category_color)
        .bind(&row.title)
        .bind(&row.description)
        .bind(&row.location)
        .bind(row.starts_at)
        .bind(row.ends_at)
        .bind(row.budget)
        .bind(&row.status)
        .bind(row.created_at)
        .bind(row.updated_at)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

async fn delete_event_projection_rows(
    tx: &mut Transaction<'_, Postgres>,
    event_id: Uuid,
) -> Result<(), WorkerError> {
    sqlx::query("DELETE FROM event_list_projection WHERE event_id = $1")
        .bind(event_id)
        .execute(&mut **tx)
        .await?;
    sqlx::query("DELETE FROM calendar_projection WHERE event_id = $1")
        .bind(event_id)
        .execute(&mut **tx)
        .await?;
    sqlx::query("DELETE FROM report_projection WHERE event_id = $1")
        .bind(event_id)
        .execute(&mut **tx)
        .await?;

    Ok(())
}

async fn update_category_projection_rows(
    tx: &mut Transaction<'_, Postgres>,
    payload: &CategoryPayload,
) -> Result<(), WorkerError> {
    sqlx::query(
        r#"
        UPDATE event_list_projection
        SET category_name = $1, category_color = $2, updated_at = GREATEST(updated_at, $5)
        WHERE user_id = $3 AND category_id = $4
        "#,
    )
    .bind(&payload.name)
    .bind(&payload.color)
    .bind(payload.user_id)
    .bind(payload.category_id)
    .bind(payload.updated_at)
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        r#"
        UPDATE report_projection
        SET category_name = $1, category_color = $2, updated_at = GREATEST(updated_at, $5)
        WHERE user_id = $3 AND category_id = $4
        "#,
    )
    .bind(&payload.name)
    .bind(&payload.color)
    .bind(payload.user_id)
    .bind(payload.category_id)
    .bind(payload.updated_at)
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        r#"
        UPDATE calendar_projection c
        SET category_color = $1, updated_at = GREATEST(c.updated_at, $4)
        FROM events e
        WHERE c.event_id = e.id
          AND e.user_id = $2
          AND e.category_id = $3
        "#,
    )
    .bind(&payload.color)
    .bind(payload.user_id)
    .bind(payload.category_id)
    .bind(payload.updated_at)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn refresh_dashboard_projection(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
) -> Result<(), WorkerError> {
    sqlx::query(
        r#"
        INSERT INTO dashboard_projection (
            user_id,
            total_events,
            upcoming_events,
            completed_events,
            cancelled_events,
            total_budget,
            updated_at
        )
        SELECT
            u.id,
            COUNT(e.id)::BIGINT,
            COUNT(*) FILTER (WHERE e.starts_at >= NOW() AND e.status <> 'cancelled')::BIGINT,
            COUNT(*) FILTER (WHERE e.status = 'completed')::BIGINT,
            COUNT(*) FILTER (WHERE e.status = 'cancelled')::BIGINT,
            COALESCE(SUM(e.budget), 0)::DOUBLE PRECISION,
            NOW()
        FROM users u
        LEFT JOIN events e ON e.user_id = u.id
        WHERE u.id = $1
        GROUP BY u.id
        ON CONFLICT (user_id) DO UPDATE
        SET
            total_events = EXCLUDED.total_events,
            upcoming_events = EXCLUDED.upcoming_events,
            completed_events = EXCLUDED.completed_events,
            cancelled_events = EXCLUDED.cancelled_events,
            total_budget = EXCLUDED.total_budget,
            updated_at = EXCLUDED.updated_at
        "#,
    )
    .bind(user_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn insert_activity(
    tx: &mut Transaction<'_, Postgres>,
    source_message_id: Uuid,
    user_id: Uuid,
    entity_type: &str,
    entity_id: Uuid,
    action: &str,
    title: &str,
    occurred_at: DateTime<Utc>,
) -> Result<(), WorkerError> {
    sqlx::query(
        r#"
        INSERT INTO recent_activity_projection (
            source_message_id,
            user_id,
            entity_type,
            entity_id,
            action,
            title,
            occurred_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (source_message_id) DO NOTHING
        "#,
    )
    .bind(source_message_id)
    .bind(user_id)
    .bind(entity_type)
    .bind(entity_id)
    .bind(action)
    .bind(title)
    .bind(occurred_at)
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        r#"
        DELETE FROM recent_activity_projection
        WHERE user_id = $1
          AND id IN (
              SELECT id
              FROM recent_activity_projection
              WHERE user_id = $1
              ORDER BY occurred_at DESC
              OFFSET 50
          )
        "#,
    )
    .bind(user_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn invalidate_dashboard_cache(redis: &redis::Client, user_id: Uuid) {
    match redis.get_multiplexed_tokio_connection().await {
        Ok(mut connection) => {
            let _: redis::RedisResult<usize> =
                connection.del(dashboard_key(&user_id.to_string())).await;
        }
        Err(error) => tracing::warn!(
            "could not connect to redis for cache invalidation: {}",
            error
        ),
    }
}

async fn acquire_export_lock(
    redis: &redis::Client,
    export_id: Uuid,
) -> Result<Option<ExportLock>, WorkerError> {
    let key = format!("export-lock:{export_id}");
    let token = Uuid::new_v4().to_string();
    let mut connection = redis.get_multiplexed_tokio_connection().await?;
    let result: Option<String> = redis::cmd("SET")
        .arg(&key)
        .arg(&token)
        .arg("EX")
        .arg(600)
        .arg("NX")
        .query_async(&mut connection)
        .await?;

    Ok(result.map(|_| ExportLock { key, token }))
}

async fn release_export_lock(redis: &redis::Client, export_lock: ExportLock) {
    let script = Script::new(
        "if redis.call('get', KEYS[1]) == ARGV[1] then return redis.call('del', KEYS[1]) else return 0 end",
    );
    match redis.get_multiplexed_tokio_connection().await {
        Ok(mut connection) => {
            let _: redis::RedisResult<i32> = script
                .key(export_lock.key)
                .arg(export_lock.token)
                .invoke_async(&mut connection)
                .await;
        }
        Err(error) => tracing::warn!("could not release export lock: {}", error),
    }
}

async fn claim_export_job(
    pool: &PgPool,
    export_id: Uuid,
    message_id: Uuid,
) -> Result<ExportClaim, WorkerError> {
    let mut tx = pool.begin().await?;
    let job = sqlx::query_as::<_, ExportJobRow>(
        r#"
        SELECT
            id,
            user_id,
            report_type,
            format,
            status,
            filters,
            object_key,
            content_type,
            error_message,
            created_at,
            started_at,
            updated_at,
            finished_at
        FROM export_jobs
        WHERE id = $1
        FOR UPDATE
        "#,
    )
    .bind(export_id)
    .fetch_optional(&mut *tx)
    .await?;

    let Some(job) = job else {
        let _ = insert_processed_message(&mut tx, EXPORT_CONSUMER, message_id).await?;
        tx.commit().await?;
        return Ok(ExportClaim::Missing);
    };

    if matches!(job.status.as_str(), "completed" | "failed") {
        let _ = insert_processed_message(&mut tx, EXPORT_CONSUMER, message_id).await?;
        tx.commit().await?;
        return Ok(ExportClaim::AlreadyProcessed);
    }

    if job.status == "processing" && !is_stale_export_job(&job) {
        tx.commit().await?;
        return Ok(ExportClaim::InProgress);
    }

    let started_event_type = if job.status == "queued" {
        Some("export.started")
    } else {
        None
    };
    let updated_job = sqlx::query_as::<_, ExportJobRow>(
        r#"
        UPDATE export_jobs
        SET status = 'processing', started_at = NOW(), updated_at = NOW(), error_message = NULL
        WHERE id = $1
        RETURNING
            id,
            user_id,
            report_type,
            format,
            status,
            filters,
            object_key,
            content_type,
            error_message,
            created_at,
            started_at,
            updated_at,
            finished_at
        "#,
    )
    .bind(export_id)
    .fetch_one(&mut *tx)
    .await?;

    if let Some(event_type) = started_event_type {
        insert_export_outbox(&mut tx, updated_job.id, event_type, &updated_job).await?;
    }

    tx.commit().await?;
    Ok(ExportClaim::Claimed(updated_job))
}

async fn mark_export_completed(
    pool: &PgPool,
    job: &ExportJobRow,
    artifact: &ExportArtifact,
    message_id: Uuid,
) -> Result<(), WorkerError> {
    let mut tx = pool.begin().await?;
    let updated = sqlx::query_as::<_, ExportJobRow>(
        r#"
        UPDATE export_jobs
        SET
            status = 'completed',
            object_key = $2,
            content_type = $3,
            error_message = NULL,
            updated_at = NOW(),
            finished_at = NOW()
        WHERE id = $1 AND status = 'processing'
        RETURNING
            id,
            user_id,
            report_type,
            format,
            status,
            filters,
            object_key,
            content_type,
            error_message,
            created_at,
            started_at,
            updated_at,
            finished_at
        "#,
    )
    .bind(job.id)
    .bind(&artifact.object_key)
    .bind(artifact.content_type)
    .fetch_optional(&mut *tx)
    .await?;

    let Some(updated) = updated else {
        let existing = load_export_job_in_tx(&mut tx, job.id).await?;
        if matches!(
            existing.as_ref().map(|row| row.status.as_str()),
            Some("completed" | "failed")
        ) {
            let _ = insert_processed_message(&mut tx, EXPORT_CONSUMER, message_id).await?;
            tx.commit().await?;
            return Ok(());
        }

        return Err(WorkerError::Internal(
            "Export job could not be marked as completed".to_string(),
        ));
    };

    insert_export_outbox(&mut tx, updated.id, "export.completed", &updated).await?;
    let _ = insert_processed_message(&mut tx, EXPORT_CONSUMER, message_id).await?;
    tx.commit().await?;

    Ok(())
}

async fn mark_export_failed(
    pool: &PgPool,
    export_id: Uuid,
    error_message: &str,
    message_id: Uuid,
) -> Result<(), WorkerError> {
    let mut tx = pool.begin().await?;
    let updated = sqlx::query_as::<_, ExportJobRow>(
        r#"
        UPDATE export_jobs
        SET
            status = 'failed',
            error_message = $2,
            updated_at = NOW(),
            finished_at = NOW()
        WHERE id = $1 AND status IN ('queued', 'processing')
        RETURNING
            id,
            user_id,
            report_type,
            format,
            status,
            filters,
            object_key,
            content_type,
            error_message,
            created_at,
            started_at,
            updated_at,
            finished_at
        "#,
    )
    .bind(export_id)
    .bind(error_message)
    .fetch_optional(&mut *tx)
    .await?;

    if let Some(updated) = updated {
        insert_export_outbox(&mut tx, updated.id, "export.failed", &updated).await?;
    } else {
        let existing = load_export_job_in_tx(&mut tx, export_id).await?;
        if !matches!(
            existing.as_ref().map(|row| row.status.as_str()),
            Some("completed" | "failed")
        ) {
            return Err(WorkerError::Internal(
                "Export job could not be marked as failed".to_string(),
            ));
        }
    }

    let _ = insert_processed_message(&mut tx, EXPORT_CONSUMER, message_id).await?;
    tx.commit().await?;

    Ok(())
}

async fn load_export_job_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    export_id: Uuid,
) -> Result<Option<ExportJobRow>, WorkerError> {
    let job = sqlx::query_as::<_, ExportJobRow>(
        r#"
        SELECT
            id,
            user_id,
            report_type,
            format,
            status,
            filters,
            object_key,
            content_type,
            error_message,
            created_at,
            started_at,
            updated_at,
            finished_at
        FROM export_jobs
        WHERE id = $1
        "#,
    )
    .bind(export_id)
    .fetch_optional(&mut **tx)
    .await?;

    Ok(job)
}

#[cfg(test)]
async fn load_export_job(
    pool: &PgPool,
    export_id: Uuid,
) -> Result<Option<ExportJobRow>, WorkerError> {
    let job = sqlx::query_as::<_, ExportJobRow>(
        r#"
        SELECT
            id,
            user_id,
            report_type,
            format,
            status,
            filters,
            object_key,
            content_type,
            error_message,
            created_at,
            started_at,
            updated_at,
            finished_at
        FROM export_jobs
        WHERE id = $1
        "#,
    )
    .bind(export_id)
    .fetch_optional(pool)
    .await?;

    Ok(job)
}

fn is_stale_export_job(job: &ExportJobRow) -> bool {
    let Some(started_at) = job.started_at else {
        return true;
    };

    started_at <= Utc::now() - chrono::Duration::minutes(EXPORT_PROCESSING_TIMEOUT_MINUTES)
}

async fn insert_export_outbox(
    tx: &mut Transaction<'_, Postgres>,
    export_id: Uuid,
    event_type: &str,
    job: &ExportJobRow,
) -> Result<(), WorkerError> {
    let payload = serde_json::to_value(ExportPayload {
        export_id: job.id,
        user_id: job.user_id,
        report_type: job.report_type.clone(),
        format: job.format.clone(),
        status: job.status.clone(),
        filters: job.filters.clone(),
        object_key: job.object_key.clone(),
        error_message: job.error_message.clone(),
        created_at: job.created_at,
        started_at: job.started_at,
        finished_at: job.finished_at,
    })?;

    sqlx::query(
        r#"
        INSERT INTO outbox_events (aggregate_type, aggregate_id, event_type, event_version, payload_json)
        VALUES ('export', $1, $2, 1, $3)
        "#,
    )
    .bind(export_id)
    .bind(event_type)
    .bind(payload)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn generate_report_summary(
    pool: &PgPool,
    user_id: Uuid,
    filters: &ExportFilters,
) -> Result<ReportSummary, WorkerError> {
    let mut builder = QueryBuilder::<Postgres>::new(
        r#"
        SELECT
            event_id AS id,
            user_id,
            category_id,
            category_name,
            category_color,
            title,
            description,
            location,
            starts_at,
            ends_at,
            budget,
            status,
            created_at,
            updated_at
        FROM report_projection
        WHERE user_id = "#,
    );
    builder.push_bind(user_id);
    push_report_filters(&mut builder, filters);
    builder.push(" ORDER BY starts_at ASC");

    let events = builder
        .build_query_as::<ReportEventRow>()
        .fetch_all(pool)
        .await?;
    let total_events = events.len();
    let total_budget = events.iter().map(|event| event.budget).sum::<f64>();

    Ok(ReportSummary {
        period_start: filters.start_date,
        period_end: filters.end_date,
        total_events,
        total_budget,
        events,
    })
}

fn push_report_filters<'a>(builder: &mut QueryBuilder<'a, Postgres>, filters: &'a ExportFilters) {
    if let Some(status) = filters
        .status
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        builder.push(" AND report_projection.status = ");
        builder.push_bind(status.to_string());
    }

    if let Some(category_id) = filters.category_id {
        builder.push(" AND report_projection.category_id = ");
        builder.push_bind(category_id);
    }

    if let Some(start_date) = filters.start_date {
        builder.push(" AND report_projection.starts_at >= ");
        builder.push_bind(start_of_day(start_date));
    }

    if let Some(end_date) = filters.end_date {
        builder.push(" AND report_projection.starts_at < ");
        builder.push_bind(end_of_day_exclusive(end_date));
    }
}

fn start_of_day(value: NaiveDate) -> DateTime<Utc> {
    DateTime::<Utc>::from_naive_utc_and_offset(value.and_hms_opt(0, 0, 0).expect("valid date"), Utc)
}

fn end_of_day_exclusive(value: NaiveDate) -> DateTime<Utc> {
    let next_day = value.succ_opt().unwrap_or(value);
    DateTime::<Utc>::from_naive_utc_and_offset(
        next_day.and_hms_opt(0, 0, 0).expect("valid date"),
        Utc,
    )
}

fn build_export_artifact(
    job: &ExportJobRow,
    summary: &ReportSummary,
) -> Result<ExportArtifact, WorkerError> {
    let object_key = format!("/exports/{}/{}.{}", job.user_id, job.id, job.format);

    match job.format.as_str() {
        "pdf" => Ok(ExportArtifact {
            object_key,
            content_type: "application/pdf",
            bytes: build_pdf(summary)?,
        }),
        "xlsx" => Ok(ExportArtifact {
            object_key,
            content_type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            bytes: build_xlsx(summary)?,
        }),
        _ => Err(WorkerError::Internal(
            "Unsupported export format".to_string(),
        )),
    }
}

fn build_pdf(summary: &ReportSummary) -> Result<Vec<u8>, WorkerError> {
    let (document, first_page, first_layer) =
        PdfDocument::new("EventDesign event report", Mm(210.0), Mm(297.0), "Layer 1");
    let font = document
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|error| WorkerError::Internal(error.to_string()))?;

    let mut y = 285.0;
    let mut page = first_page;
    let mut layer = first_layer;
    let mut current_layer = document.get_page(page).get_layer(layer);
    let mut page_number = 1;

    for line in pdf_lines(summary) {
        if y < 14.0 {
            page_number += 1;
            let (next_page, next_layer) =
                document.add_page(Mm(210.0), Mm(297.0), format!("Layer {page_number}"));
            page = next_page;
            layer = next_layer;
            current_layer = document.get_page(page).get_layer(layer);
            y = 285.0;
        }

        current_layer.use_text(line, 11.0, Mm(12.0), Mm(y), &font);
        y -= 7.0;
    }

    document
        .save_to_bytes()
        .map_err(|error| WorkerError::Internal(error.to_string()))
}

fn pdf_lines(summary: &ReportSummary) -> Vec<String> {
    let mut lines = vec![
        "EventDesign Event Report".to_string(),
        format!("Total events: {}", summary.total_events),
        format!("Total budget: {:.2}", summary.total_budget),
        format!(
            "Period: {} - {}",
            summary
                .period_start
                .map(|value| value.to_string())
                .unwrap_or_else(|| "Any".to_string()),
            summary
                .period_end
                .map(|value| value.to_string())
                .unwrap_or_else(|| "Any".to_string())
        ),
        String::new(),
        "Events".to_string(),
    ];

    for event in &summary.events {
        lines.push(format!(
            "{} | {} | {} | {}",
            event.starts_at.format("%Y-%m-%d %H:%M"),
            event.title,
            event.category_name,
            event.status
        ));
        lines.push(format!(
            "Location: {} | Ends: {} | Budget: {:.2}",
            if event.location.trim().is_empty() {
                "Not specified"
            } else {
                event.location.as_str()
            },
            event.ends_at.format("%Y-%m-%d %H:%M"),
            event.budget
        ));
        lines.push(String::new());
    }

    lines
}

fn build_xlsx(summary: &ReportSummary) -> Result<Vec<u8>, WorkerError> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet
        .write_string(0, 0, "Title")
        .map_err(map_xlsx_error)?;
    worksheet
        .write_string(0, 1, "Category")
        .map_err(map_xlsx_error)?;
    worksheet
        .write_string(0, 2, "Location")
        .map_err(map_xlsx_error)?;
    worksheet
        .write_string(0, 3, "Status")
        .map_err(map_xlsx_error)?;
    worksheet
        .write_string(0, 4, "Starts At")
        .map_err(map_xlsx_error)?;
    worksheet
        .write_string(0, 5, "Ends At")
        .map_err(map_xlsx_error)?;
    worksheet
        .write_string(0, 6, "Budget")
        .map_err(map_xlsx_error)?;

    for (index, event) in summary.events.iter().enumerate() {
        let row = (index + 1) as u32;
        worksheet
            .write_string(row, 0, &event.title)
            .map_err(map_xlsx_error)?;
        worksheet
            .write_string(row, 1, &event.category_name)
            .map_err(map_xlsx_error)?;
        worksheet
            .write_string(row, 2, &event.location)
            .map_err(map_xlsx_error)?;
        worksheet
            .write_string(row, 3, &event.status)
            .map_err(map_xlsx_error)?;
        worksheet
            .write_string(row, 4, event.starts_at.format("%Y-%m-%d %H:%M").to_string())
            .map_err(map_xlsx_error)?;
        worksheet
            .write_string(row, 5, event.ends_at.format("%Y-%m-%d %H:%M").to_string())
            .map_err(map_xlsx_error)?;
        worksheet
            .write_number(row, 6, event.budget)
            .map_err(map_xlsx_error)?;
    }

    workbook.save_to_buffer().map_err(map_xlsx_error)
}

fn map_xlsx_error(error: rust_xlsxwriter::XlsxError) -> WorkerError {
    WorkerError::Internal(error.to_string())
}

async fn upload_export_artifact(
    bucket: &Bucket,
    artifact: &ExportArtifact,
) -> Result<(), WorkerError> {
    let response = bucket
        .put_object_with_content_type(&artifact.object_key, &artifact.bytes, artifact.content_type)
        .await
        .map_err(|error| {
            WorkerError::Internal(format!("Could not upload export artifact: {error}"))
        })?;

    if !(200..300).contains(&response.status_code()) {
        return Err(WorkerError::Internal(format!(
            "MinIO upload failed with status {}",
            response.status_code()
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;

    async fn insert_user(pool: &PgPool) -> Uuid {
        let user_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO users (id, email, password_hash, full_name)
            VALUES ($1, 'worker@eventdesign.local', 'hash', 'Worker User')
            "#,
        )
        .bind(user_id)
        .execute(pool)
        .await
        .unwrap();
        sqlx::query(
            r#"
            INSERT INTO ui_settings (user_id, theme)
            VALUES ($1, 'system')
            "#,
        )
        .bind(user_id)
        .execute(pool)
        .await
        .unwrap();
        user_id
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn projection_helpers_refresh_read_models(pool: PgPool) {
        let user_id = insert_user(&pool).await;
        let category_id = Uuid::new_v4();
        let event_id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO categories (id, user_id, name, color)
            VALUES ($1, $2, 'Conference', '#0f766e')
            "#,
        )
        .bind(category_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            r#"
            INSERT INTO events (
                id,
                user_id,
                category_id,
                title,
                description,
                location,
                starts_at,
                ends_at,
                budget,
                status
            )
            VALUES (
                $1,
                $2,
                $3,
                'Defense rehearsal',
                'dry run',
                'Room 301',
                '2026-03-15T10:00:00Z',
                '2026-03-15T12:00:00Z',
                850.0,
                'planned'
            )
            "#,
        )
        .bind(event_id)
        .bind(user_id)
        .bind(category_id)
        .execute(&pool)
        .await
        .unwrap();

        let mut tx = pool.begin().await.unwrap();
        upsert_event_projection_rows(&mut tx, event_id)
            .await
            .expect("projection upsert succeeds");
        refresh_dashboard_projection(&mut tx, user_id)
            .await
            .expect("dashboard refresh succeeds");
        tx.commit().await.unwrap();

        let projection_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM event_list_projection WHERE event_id = $1",
        )
        .bind(event_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        let total_events = sqlx::query_scalar::<_, i64>(
            "SELECT total_events FROM dashboard_projection WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(projection_count, 1);
        assert_eq!(total_events, 1);
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn projection_refresh_is_idempotent_for_redelivery(pool: PgPool) {
        let user_id = insert_user(&pool).await;
        let category_id = Uuid::new_v4();
        let event_id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO categories (id, user_id, name, color)
            VALUES ($1, $2, 'Conference', '#0f766e')
            "#,
        )
        .bind(category_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            r#"
            INSERT INTO events (
                id,
                user_id,
                category_id,
                title,
                description,
                location,
                starts_at,
                ends_at,
                budget,
                status
            )
            VALUES (
                $1,
                $2,
                $3,
                'Idempotent projection',
                'redelivery safety',
                'Room 302',
                '2026-03-16T10:00:00Z',
                '2026-03-16T12:00:00Z',
                640.0,
                'planned'
            )
            "#,
        )
        .bind(event_id)
        .bind(user_id)
        .bind(category_id)
        .execute(&pool)
        .await
        .unwrap();

        for _ in 0..2 {
            let mut tx = pool.begin().await.unwrap();
            upsert_event_projection_rows(&mut tx, event_id)
                .await
                .expect("projection upsert succeeds");
            refresh_dashboard_projection(&mut tx, user_id)
                .await
                .expect("dashboard refresh succeeds");
            tx.commit().await.unwrap();
        }

        let list_projection_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM event_list_projection WHERE event_id = $1",
        )
        .bind(event_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        let calendar_projection_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM calendar_projection WHERE event_id = $1",
        )
        .bind(event_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        let report_projection_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM report_projection WHERE event_id = $1",
        )
        .bind(event_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        let total_events = sqlx::query_scalar::<_, i64>(
            "SELECT total_events FROM dashboard_projection WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(list_projection_count, 1);
        assert_eq!(calendar_projection_count, 1);
        assert_eq!(report_projection_count, 1);
        assert_eq!(total_events, 1);
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn export_job_lifecycle_updates_status(pool: PgPool) {
        let user_id = insert_user(&pool).await;
        let export_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO export_jobs (id, user_id, report_type, format, status, filters)
            VALUES ($1, $2, 'summary', 'pdf', 'queued', '{}'::jsonb)
            "#,
        )
        .bind(export_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

        let processing_job = match claim_export_job(&pool, export_id, message_id)
            .await
            .expect("claim succeeds")
        {
            ExportClaim::Claimed(job) => job,
            _ => panic!("export job should have been claimed"),
        };
        assert_eq!(processing_job.status, "processing");

        let artifact = ExportArtifact {
            object_key: "/exports/demo/report.pdf".to_string(),
            content_type: "application/pdf",
            bytes: vec![1, 2, 3],
        };
        mark_export_completed(&pool, &processing_job, &artifact, message_id)
            .await
            .expect("processing job becomes completed");
        let completed_job = load_export_job(&pool, export_id).await.unwrap().unwrap();
        let processed = processed_message_exists(&pool, EXPORT_CONSUMER, message_id)
            .await
            .unwrap();
        let lifecycle_events = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM outbox_events WHERE aggregate_id = $1 AND aggregate_type = 'export'",
        )
        .bind(export_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(completed_job.status, "completed");
        assert_eq!(
            completed_job.object_key.as_deref(),
            Some("/exports/demo/report.pdf")
        );
        assert!(processed);
        assert_eq!(lifecycle_events, 2);
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn export_claim_skips_active_processing_and_retries_stale_jobs(pool: PgPool) {
        let user_id = insert_user(&pool).await;
        let export_id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO export_jobs (
                id,
                user_id,
                report_type,
                format,
                status,
                filters,
                started_at,
                updated_at
            )
            VALUES (
                $1,
                $2,
                'summary',
                'pdf',
                'processing',
                '{}'::jsonb,
                NOW(),
                NOW()
            )
            "#,
        )
        .bind(export_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

        let active_claim = claim_export_job(&pool, export_id, Uuid::new_v4())
            .await
            .expect("active claim check succeeds");
        assert!(matches!(active_claim, ExportClaim::InProgress));

        sqlx::query(
            r#"
            UPDATE export_jobs
            SET started_at = NOW() - INTERVAL '20 minutes', updated_at = NOW() - INTERVAL '20 minutes'
            WHERE id = $1
            "#,
        )
        .bind(export_id)
        .execute(&pool)
        .await
        .unwrap();

        let stale_claim = claim_export_job(&pool, export_id, Uuid::new_v4())
            .await
            .expect("stale claim check succeeds");
        assert!(matches!(stale_claim, ExportClaim::Claimed(_)));
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn processed_message_claim_is_idempotent(pool: PgPool) {
        let message_id = Uuid::new_v4();
        let mut tx = pool.begin().await.unwrap();

        let first = claim_processed_message(&mut tx, PROJECTION_CONSUMER, message_id)
            .await
            .unwrap();
        let second = claim_processed_message(&mut tx, PROJECTION_CONSUMER, message_id)
            .await
            .unwrap();
        tx.commit().await.unwrap();

        assert!(first);
        assert!(!second);
    }
}
