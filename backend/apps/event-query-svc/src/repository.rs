use chrono::{NaiveDate, TimeZone, Utc};
use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::models::{
    ActivityRow, CalendarItemRow, CategoryRow, DashboardProjectionRow, EventFilters, EventItemRow,
};

pub async fn list_categories(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<CategoryRow>, sqlx::Error> {
    sqlx::query_as::<_, CategoryRow>(
        r#"
        SELECT id, user_id, name, color, created_at, updated_at
        FROM categories
        WHERE user_id = $1
        ORDER BY name ASC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn list_events(
    pool: &PgPool,
    user_id: Uuid,
    filters: &EventFilters,
) -> Result<Vec<EventItemRow>, sqlx::Error> {
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
        FROM event_list_projection
        WHERE user_id = "#,
    );
    builder.push_bind(user_id);
    push_projection_filters(&mut builder, "event_list_projection", filters, "starts_at");
    push_projection_order(&mut builder, filters, "starts_at");

    builder
        .build_query_as::<EventItemRow>()
        .fetch_all(pool)
        .await
}

pub async fn get_event(
    pool: &PgPool,
    user_id: Uuid,
    event_id: Uuid,
) -> Result<Option<EventItemRow>, sqlx::Error> {
    sqlx::query_as::<_, EventItemRow>(
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
        FROM event_list_projection
        WHERE user_id = $1 AND event_id = $2
        "#,
    )
    .bind(user_id)
    .bind(event_id)
    .fetch_optional(pool)
    .await
}

pub async fn get_calendar(
    pool: &PgPool,
    user_id: Uuid,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Vec<CalendarItemRow>, sqlx::Error> {
    sqlx::query_as::<_, CalendarItemRow>(
        r#"
        SELECT
            event_id,
            title,
            date_bucket AS date,
            starts_at,
            ends_at,
            status,
            category_color
        FROM calendar_projection
        WHERE user_id = $1
          AND date_bucket >= $2
          AND date_bucket <= $3
        ORDER BY starts_at ASC
        "#,
    )
    .bind(user_id)
    .bind(start_date)
    .bind(end_date)
    .fetch_all(pool)
    .await
}

pub async fn get_dashboard_projection(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<DashboardProjectionRow>, sqlx::Error> {
    sqlx::query_as::<_, DashboardProjectionRow>(
        r#"
        SELECT user_id, total_events, upcoming_events, completed_events, cancelled_events, total_budget, updated_at
        FROM dashboard_projection
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn list_upcoming_events(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
) -> Result<Vec<EventItemRow>, sqlx::Error> {
    sqlx::query_as::<_, EventItemRow>(
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
        FROM event_list_projection
        WHERE user_id = $1
          AND starts_at >= NOW()
          AND status <> 'cancelled'
        ORDER BY starts_at ASC
        LIMIT $2
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn list_recent_activity(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
) -> Result<Vec<ActivityRow>, sqlx::Error> {
    sqlx::query_as::<_, ActivityRow>(
        r#"
        SELECT id, entity_type, entity_id, action, title, occurred_at
        FROM recent_activity_projection
        WHERE user_id = $1
        ORDER BY occurred_at DESC
        LIMIT $2
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn list_report_rows(
    pool: &PgPool,
    user_id: Uuid,
    filters: &EventFilters,
) -> Result<Vec<EventItemRow>, sqlx::Error> {
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
    push_projection_filters(&mut builder, "report_projection", filters, "starts_at");
    push_projection_order(&mut builder, filters, "starts_at");

    builder
        .build_query_as::<EventItemRow>()
        .fetch_all(pool)
        .await
}

fn push_projection_filters<'a>(
    builder: &mut QueryBuilder<'a, Postgres>,
    table_name: &str,
    filters: &'a EventFilters,
    date_column: &str,
) {
    if let Some(search) = filters
        .search
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let pattern = format!("%{}%", search.to_lowercase());
        builder.push(format!(" AND (LOWER({table_name}.title) LIKE "));
        builder.push_bind(pattern.clone());
        builder.push(format!(" OR LOWER({table_name}.description) LIKE "));
        builder.push_bind(pattern.clone());
        builder.push(format!(" OR LOWER({table_name}.location) LIKE "));
        builder.push_bind(pattern);
        builder.push(")");
    }

    if let Some(status) = filters
        .status
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        builder.push(format!(" AND {table_name}.status = "));
        builder.push_bind(status.to_string());
    }

    if let Some(category_id) = filters.category_id {
        builder.push(format!(" AND {table_name}.category_id = "));
        builder.push_bind(category_id);
    }

    if let Some(start_date) = filters.start_date {
        builder.push(format!(" AND {table_name}.{date_column} >= "));
        builder.push_bind(start_of_day(start_date));
    }

    if let Some(end_date) = filters.end_date {
        builder.push(format!(" AND {table_name}.{date_column} < "));
        builder.push_bind(end_of_day_exclusive(end_date));
    }
}

fn push_projection_order(
    builder: &mut QueryBuilder<'_, Postgres>,
    filters: &EventFilters,
    default_column: &str,
) {
    let sort_column = match filters.sort_by.as_deref() {
        Some("starts_at") => "starts_at",
        Some("title") => "title",
        Some("budget") => "budget",
        Some("status") => "status",
        Some("updated_at") => "updated_at",
        Some("ends_at") => "ends_at",
        Some("category_name") => "category_name",
        _ => default_column,
    };
    let sort_direction = match filters.sort_dir.as_deref() {
        Some("desc") => "DESC",
        _ => "ASC",
    };

    builder.push(format!(
        " ORDER BY {sort_column} {sort_direction}, starts_at ASC, event_id ASC"
    ));
}

fn start_of_day(value: NaiveDate) -> chrono::DateTime<Utc> {
    Utc.from_utc_datetime(&value.and_hms_opt(0, 0, 0).expect("valid date"))
}

fn end_of_day_exclusive(value: NaiveDate) -> chrono::DateTime<Utc> {
    let next_day = value.succ_opt().unwrap_or(value);
    Utc.from_utc_datetime(&next_day.and_hms_opt(0, 0, 0).expect("valid date"))
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;

    #[sqlx::test(migrations = "../../migrations")]
    async fn report_rows_apply_filters_and_sorting(pool: PgPool) {
        let user_id = Uuid::new_v4();
        let category_id = Uuid::new_v4();
        let other_category_id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO users (id, email, password_hash, full_name)
            VALUES ($1, 'report@eventdesign.local', 'hash', 'Report User')
            "#,
        )
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            r#"
            INSERT INTO ui_settings (user_id, theme)
            VALUES ($1, 'system')
            "#,
        )
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();
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
            VALUES
                ($1, $3, $4, 'Conference', '#0f766e', 'Alpha', 'a', 'Room A', '2026-03-10T10:00:00Z', '2026-03-10T11:00:00Z', 250.0, 'planned', NOW(), NOW()),
                ($2, $3, $5, 'Launch', '#be123c', 'Omega', 'b', 'Room B', '2026-03-12T10:00:00Z', '2026-03-12T11:00:00Z', 900.0, 'planned', NOW(), NOW())
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(category_id)
        .bind(other_category_id)
        .execute(&pool)
        .await
        .unwrap();

        let rows = list_report_rows(
            &pool,
            user_id,
            &EventFilters {
                search: None,
                status: Some("planned".to_string()),
                category_id: None,
                start_date: None,
                end_date: None,
                sort_by: Some("budget".to_string()),
                sort_dir: Some("desc".to_string()),
            },
        )
        .await
        .unwrap();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].title, "Omega");
        assert!(rows[0].budget > rows[1].budget);
    }
}
