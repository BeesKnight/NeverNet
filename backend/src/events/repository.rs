use chrono::{NaiveDate, TimeZone, Utc};
use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::events::models::{
    CreateEventRequest, Event, EventFilters, EventListItem, UpdateEventRequest,
};

pub async fn list(
    pool: &PgPool,
    user_id: Uuid,
    filters: &EventFilters,
) -> Result<Vec<EventListItem>, sqlx::Error> {
    let mut builder = QueryBuilder::<Postgres>::new(
        r#"
        SELECT
            e.id,
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
        WHERE e.user_id = "#,
    );
    builder.push_bind(user_id);

    push_filters(&mut builder, filters);
    builder.push(" ORDER BY e.starts_at ASC");

    builder
        .build_query_as::<EventListItem>()
        .fetch_all(pool)
        .await
}

pub async fn find_by_id(
    pool: &PgPool,
    user_id: Uuid,
    event_id: Uuid,
) -> Result<Option<Event>, sqlx::Error> {
    sqlx::query_as::<_, Event>(
        r#"
        SELECT id, user_id, category_id, title, description, location, starts_at, ends_at, budget, status, created_at, updated_at
        FROM events
        WHERE user_id = $1 AND id = $2
        "#,
    )
    .bind(user_id)
    .bind(event_id)
    .fetch_optional(pool)
    .await
}

pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    payload: &CreateEventRequest,
) -> Result<Event, sqlx::Error> {
    sqlx::query_as::<_, Event>(
        r#"
        INSERT INTO events (user_id, category_id, title, description, location, starts_at, ends_at, budget, status)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, user_id, category_id, title, description, location, starts_at, ends_at, budget, status, created_at, updated_at
        "#,
    )
    .bind(user_id)
    .bind(payload.category_id)
    .bind(payload.title.trim())
    .bind(payload.description.trim())
    .bind(payload.location.trim())
    .bind(payload.starts_at)
    .bind(payload.ends_at)
    .bind(payload.budget)
    .bind(payload.status.as_str())
    .fetch_one(pool)
    .await
}

pub async fn update(
    pool: &PgPool,
    user_id: Uuid,
    event_id: Uuid,
    payload: &UpdateEventRequest,
) -> Result<Option<Event>, sqlx::Error> {
    sqlx::query_as::<_, Event>(
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
    .bind(payload.title.trim())
    .bind(payload.description.trim())
    .bind(payload.location.trim())
    .bind(payload.starts_at)
    .bind(payload.ends_at)
    .bind(payload.budget)
    .bind(payload.status.as_str())
    .fetch_optional(pool)
    .await
}

pub async fn delete(pool: &PgPool, user_id: Uuid, event_id: Uuid) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM events
        WHERE user_id = $1 AND id = $2
        "#,
    )
    .bind(user_id)
    .bind(event_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

fn push_filters<'a>(builder: &mut QueryBuilder<'a, Postgres>, filters: &'a EventFilters) {
    if let Some(search) = filters
        .search
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let pattern = format!("%{}%", search.to_lowercase());
        builder.push(" AND (LOWER(e.title) LIKE ");
        builder.push_bind(pattern.clone());
        builder.push(" OR LOWER(e.description) LIKE ");
        builder.push_bind(pattern.clone());
        builder.push(" OR LOWER(e.location) LIKE ");
        builder.push_bind(pattern);
        builder.push(")");
    }

    if let Some(status) = filters
        .status
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        builder.push(" AND e.status = ");
        builder.push_bind(status.to_string());
    }

    if let Some(category_id) = filters.category_id {
        builder.push(" AND e.category_id = ");
        builder.push_bind(category_id);
    }

    if let Some(start_date) = filters.start_date {
        builder.push(" AND e.starts_at >= ");
        builder.push_bind(start_of_day(start_date));
    }

    if let Some(end_date) = filters.end_date {
        builder.push(" AND e.starts_at < ");
        builder.push_bind(end_of_day_exclusive(end_date));
    }
}

fn start_of_day(value: NaiveDate) -> chrono::DateTime<Utc> {
    Utc.from_utc_datetime(&value.and_hms_opt(0, 0, 0).expect("valid date"))
}

fn end_of_day_exclusive(value: NaiveDate) -> chrono::DateTime<Utc> {
    let next_day = value.succ_opt().unwrap_or(value);
    Utc.from_utc_datetime(&next_day.and_hms_opt(0, 0, 0).expect("valid date"))
}
