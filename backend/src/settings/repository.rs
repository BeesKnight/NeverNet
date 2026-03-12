use sqlx::PgPool;
use uuid::Uuid;

use crate::settings::models::UiSettings;

pub async fn get(pool: &PgPool, user_id: Uuid) -> Result<Option<UiSettings>, sqlx::Error> {
    sqlx::query_as::<_, UiSettings>(
        r#"
        SELECT user_id, theme, created_at, updated_at
        FROM ui_settings
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn upsert(pool: &PgPool, user_id: Uuid, theme: &str) -> Result<UiSettings, sqlx::Error> {
    sqlx::query_as::<_, UiSettings>(
        r#"
        INSERT INTO ui_settings (user_id, theme)
        VALUES ($1, $2)
        ON CONFLICT (user_id)
        DO UPDATE SET theme = EXCLUDED.theme, updated_at = NOW()
        RETURNING user_id, theme, created_at, updated_at
        "#,
    )
    .bind(user_id)
    .bind(theme)
    .fetch_one(pool)
    .await
}
