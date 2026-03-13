use serde_json::Value;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::models::ExportJob;

pub async fn create_export_job(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    report_type: &str,
    format: &str,
    filters: Value,
) -> Result<ExportJob, sqlx::Error> {
    sqlx::query_as::<_, ExportJob>(
        r#"
        INSERT INTO export_jobs (user_id, report_type, format, status, filters)
        VALUES ($1, $2, $3, 'queued', $4)
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
    .bind(user_id)
    .bind(report_type)
    .bind(format)
    .bind(filters)
    .fetch_one(&mut **tx)
    .await
}

pub async fn insert_outbox_event(
    tx: &mut Transaction<'_, Postgres>,
    aggregate_id: Uuid,
    event_type: &str,
    payload: Value,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO outbox_events (aggregate_type, aggregate_id, event_type, event_version, payload_json)
        VALUES ('export', $1, $2, 1, $3)
        "#,
    )
    .bind(aggregate_id)
    .bind(event_type)
    .bind(payload)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub async fn list_export_jobs(pool: &PgPool, user_id: Uuid) -> Result<Vec<ExportJob>, sqlx::Error> {
    sqlx::query_as::<_, ExportJob>(
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
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn get_export_job(
    pool: &PgPool,
    user_id: Uuid,
    export_id: Uuid,
) -> Result<Option<ExportJob>, sqlx::Error> {
    sqlx::query_as::<_, ExportJob>(
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
        WHERE user_id = $1 AND id = $2
        "#,
    )
    .bind(user_id)
    .bind(export_id)
    .fetch_optional(pool)
    .await
}
