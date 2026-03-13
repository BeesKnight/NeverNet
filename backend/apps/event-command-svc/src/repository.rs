use serde_json::Value;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::models::{Category, EventRecord};

pub async fn find_category_by_id(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    category_id: Uuid,
) -> Result<Option<Category>, sqlx::Error> {
    sqlx::query_as::<_, Category>(
        r#"
        SELECT id, user_id, name, color, created_at, updated_at
        FROM categories
        WHERE user_id = $1 AND id = $2
        "#,
    )
    .bind(user_id)
    .bind(category_id)
    .fetch_optional(&mut **tx)
    .await
}

pub async fn create_category(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    name: &str,
    color: &str,
) -> Result<Category, sqlx::Error> {
    sqlx::query_as::<_, Category>(
        r#"
        INSERT INTO categories (user_id, name, color)
        VALUES ($1, $2, $3)
        RETURNING id, user_id, name, color, created_at, updated_at
        "#,
    )
    .bind(user_id)
    .bind(name)
    .bind(color)
    .fetch_one(&mut **tx)
    .await
}

pub async fn update_category(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    category_id: Uuid,
    name: &str,
    color: &str,
) -> Result<Option<Category>, sqlx::Error> {
    sqlx::query_as::<_, Category>(
        r#"
        UPDATE categories
        SET name = $3, color = $4, updated_at = NOW()
        WHERE user_id = $1 AND id = $2
        RETURNING id, user_id, name, color, created_at, updated_at
        "#,
    )
    .bind(user_id)
    .bind(category_id)
    .bind(name)
    .bind(color)
    .fetch_optional(&mut **tx)
    .await
}

pub async fn count_events_for_category(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    category_id: Uuid,
) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)::BIGINT
        FROM events
        WHERE user_id = $1 AND category_id = $2
        "#,
    )
    .bind(user_id)
    .bind(category_id)
    .fetch_one(&mut **tx)
    .await?;

    Ok(row.0)
}

pub async fn delete_category(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    category_id: Uuid,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM categories
        WHERE user_id = $1 AND id = $2
        "#,
    )
    .bind(user_id)
    .bind(category_id)
    .execute(&mut **tx)
    .await?;

    Ok(result.rows_affected())
}

pub async fn find_event_by_id(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    event_id: Uuid,
) -> Result<Option<EventRecord>, sqlx::Error> {
    sqlx::query_as::<_, EventRecord>(
        r#"
        SELECT id, user_id, category_id, title, description, location, starts_at, ends_at, budget, status, created_at, updated_at
        FROM events
        WHERE user_id = $1 AND id = $2
        "#,
    )
    .bind(user_id)
    .bind(event_id)
    .fetch_optional(&mut **tx)
    .await
}

pub struct EventMutation<'a> {
    pub category_id: Uuid,
    pub title: &'a str,
    pub description: &'a str,
    pub location: &'a str,
    pub starts_at: chrono::DateTime<chrono::Utc>,
    pub ends_at: chrono::DateTime<chrono::Utc>,
    pub budget: f64,
    pub status: &'a str,
}

pub async fn create_event(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    payload: &EventMutation<'_>,
) -> Result<EventRecord, sqlx::Error> {
    sqlx::query_as::<_, EventRecord>(
        r#"
        INSERT INTO events (user_id, category_id, title, description, location, starts_at, ends_at, budget, status)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, user_id, category_id, title, description, location, starts_at, ends_at, budget, status, created_at, updated_at
        "#,
    )
    .bind(user_id)
    .bind(payload.category_id)
    .bind(payload.title)
    .bind(payload.description)
    .bind(payload.location)
    .bind(payload.starts_at)
    .bind(payload.ends_at)
    .bind(payload.budget)
    .bind(payload.status)
    .fetch_one(&mut **tx)
    .await
}

pub async fn update_event(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    event_id: Uuid,
    payload: &EventMutation<'_>,
) -> Result<Option<EventRecord>, sqlx::Error> {
    sqlx::query_as::<_, EventRecord>(
        r#"
        UPDATE events
        SET
            category_id = $3,
            title = $4,
            description = $5,
            location = $6,
            starts_at = $7,
            ends_at = $8,
            budget = $9,
            status = $10,
            updated_at = NOW()
        WHERE user_id = $1 AND id = $2
        RETURNING id, user_id, category_id, title, description, location, starts_at, ends_at, budget, status, created_at, updated_at
        "#,
    )
    .bind(user_id)
    .bind(event_id)
    .bind(payload.category_id)
    .bind(payload.title)
    .bind(payload.description)
    .bind(payload.location)
    .bind(payload.starts_at)
    .bind(payload.ends_at)
    .bind(payload.budget)
    .bind(payload.status)
    .fetch_optional(&mut **tx)
    .await
}

pub async fn delete_event(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    event_id: Uuid,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM events
        WHERE user_id = $1 AND id = $2
        "#,
    )
    .bind(user_id)
    .bind(event_id)
    .execute(&mut **tx)
    .await?;

    Ok(result.rows_affected())
}

pub async fn insert_outbox_event(
    tx: &mut Transaction<'_, Postgres>,
    aggregate_type: &str,
    aggregate_id: Uuid,
    event_type: &str,
    payload: Value,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO outbox_events (aggregate_type, aggregate_id, event_type, event_version, payload_json)
        VALUES ($1, $2, $3, 1, $4)
        "#,
    )
    .bind(aggregate_type)
    .bind(aggregate_id)
    .bind(event_type)
    .bind(payload)
    .execute(&mut **tx)
    .await?;

    Ok(())
}
