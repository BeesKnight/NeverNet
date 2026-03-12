use sqlx::PgPool;
use uuid::Uuid;

use crate::categories::models::Category;

pub async fn list(pool: &PgPool, user_id: Uuid) -> Result<Vec<Category>, sqlx::Error> {
    sqlx::query_as::<_, Category>(
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

pub async fn find_by_id(
    pool: &PgPool,
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
    .fetch_optional(pool)
    .await
}

pub async fn create(
    pool: &PgPool,
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
    .fetch_one(pool)
    .await
}

pub async fn update(
    pool: &PgPool,
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
    .fetch_optional(pool)
    .await
}

pub async fn delete(pool: &PgPool, user_id: Uuid, category_id: Uuid) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM categories
        WHERE user_id = $1 AND id = $2
        "#,
    )
    .bind(user_id)
    .bind(category_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

pub async fn events_count(
    pool: &PgPool,
    user_id: Uuid,
    category_id: Uuid,
) -> Result<i64, sqlx::Error> {
    let count: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)::BIGINT
        FROM events
        WHERE user_id = $1 AND category_id = $2
        "#,
    )
    .bind(user_id)
    .bind(category_id)
    .fetch_one(pool)
    .await?;

    Ok(count.0)
}
