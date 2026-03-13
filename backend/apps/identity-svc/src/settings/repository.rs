use sqlx::PgPool;
use uuid::Uuid;

use crate::settings::models::UiSettings;

pub async fn get(pool: &PgPool, user_id: Uuid) -> Result<Option<UiSettings>, sqlx::Error> {
    sqlx::query_as::<_, UiSettings>(
        r#"
        SELECT user_id, theme, accent_color, default_view, created_at, updated_at
        FROM ui_settings
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn ensure_default(pool: &PgPool, user_id: Uuid) -> Result<UiSettings, sqlx::Error> {
    sqlx::query_as::<_, UiSettings>(
        r#"
        INSERT INTO ui_settings (user_id, theme, accent_color, default_view)
        VALUES ($1, 'system', '#b6532f', 'dashboard')
        ON CONFLICT (user_id) DO UPDATE SET user_id = ui_settings.user_id
        RETURNING user_id, theme, accent_color, default_view, created_at, updated_at
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
}

pub async fn upsert(
    pool: &PgPool,
    user_id: Uuid,
    theme: &str,
    accent_color: &str,
    default_view: &str,
) -> Result<UiSettings, sqlx::Error> {
    sqlx::query_as::<_, UiSettings>(
        r#"
        INSERT INTO ui_settings (user_id, theme, accent_color, default_view)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (user_id) DO UPDATE SET
            theme = EXCLUDED.theme,
            accent_color = EXCLUDED.accent_color,
            default_view = EXCLUDED.default_view,
            updated_at = NOW()
        RETURNING user_id, theme, accent_color, default_view, created_at, updated_at
        "#,
    )
    .bind(user_id)
    .bind(theme)
    .bind(accent_color)
    .bind(default_view)
    .fetch_one(pool)
    .await
}
