use sqlx::PgPool;
use uuid::Uuid;

use crate::exports::models::ExportJob;

pub async fn list(pool: &PgPool, user_id: Uuid) -> Result<Vec<ExportJob>, sqlx::Error> {
    sqlx::query_as::<_, ExportJob>(
        r#"
        SELECT id, user_id, report_type, format, status, filters, file_path, error_message, created_at, updated_at, finished_at
        FROM export_jobs
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn find_by_id(
    pool: &PgPool,
    user_id: Uuid,
    export_id: Uuid,
) -> Result<Option<ExportJob>, sqlx::Error> {
    sqlx::query_as::<_, ExportJob>(
        r#"
        SELECT id, user_id, report_type, format, status, filters, file_path, error_message, created_at, updated_at, finished_at
        FROM export_jobs
        WHERE user_id = $1 AND id = $2
        "#,
    )
    .bind(user_id)
    .bind(export_id)
    .fetch_optional(pool)
    .await
}

pub async fn find_by_job_id(
    pool: &PgPool,
    export_id: Uuid,
) -> Result<Option<ExportJob>, sqlx::Error> {
    sqlx::query_as::<_, ExportJob>(
        r#"
        SELECT id, user_id, report_type, format, status, filters, file_path, error_message, created_at, updated_at, finished_at
        FROM export_jobs
        WHERE id = $1
        "#,
    )
    .bind(export_id)
    .fetch_optional(pool)
    .await
}

pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    report_type: &str,
    format: &str,
    filters: serde_json::Value,
) -> Result<ExportJob, sqlx::Error> {
    sqlx::query_as::<_, ExportJob>(
        r#"
        INSERT INTO export_jobs (user_id, report_type, format, status, filters)
        VALUES ($1, $2, $3, 'pending', $4)
        RETURNING id, user_id, report_type, format, status, filters, file_path, error_message, created_at, updated_at, finished_at
        "#,
    )
    .bind(user_id)
    .bind(report_type)
    .bind(format)
    .bind(filters)
    .fetch_one(pool)
    .await
}

pub async fn set_processing(pool: &PgPool, job_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE export_jobs
        SET status = 'processing', error_message = NULL, updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(job_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn complete(pool: &PgPool, job_id: Uuid, file_path: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE export_jobs
        SET status = 'completed', file_path = $2, finished_at = NOW(), updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(job_id)
    .bind(file_path)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn fail(pool: &PgPool, job_id: Uuid, error_message: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE export_jobs
        SET status = 'failed', error_message = $2, updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(job_id)
    .bind(error_message)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn pending(pool: &PgPool) -> Result<Vec<ExportJob>, sqlx::Error> {
    sqlx::query_as::<_, ExportJob>(
        r#"
        SELECT id, user_id, report_type, format, status, filters, file_path, error_message, created_at, updated_at, finished_at
        FROM export_jobs
        WHERE status IN ('pending', 'processing')
        ORDER BY created_at ASC
        "#,
    )
    .fetch_all(pool)
    .await
}
