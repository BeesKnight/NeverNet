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

#[cfg(test)]
mod tests {
    use super::*;

    async fn insert_user(pool: &PgPool, user_id: Uuid) {
        sqlx::query(
            r#"
            INSERT INTO users (id, email, password_hash, full_name)
            VALUES ($1, 'settings@eventdesign.local', 'hash', 'Settings User')
            "#,
        )
        .bind(user_id)
        .execute(pool)
        .await
        .unwrap();
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn ensure_default_creates_or_reuses_settings(pool: PgPool) {
        let user_id = Uuid::new_v4();
        insert_user(&pool, user_id).await;

        let first = ensure_default(&pool, user_id).await.unwrap();
        let second = ensure_default(&pool, user_id).await.unwrap();

        assert_eq!(first.theme, "system");
        assert_eq!(first.accent_color, "#b6532f");
        assert_eq!(first.default_view, "dashboard");
        assert_eq!(second.user_id, user_id);
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn upsert_updates_existing_settings(pool: PgPool) {
        let user_id = Uuid::new_v4();
        insert_user(&pool, user_id).await;
        ensure_default(&pool, user_id).await.unwrap();

        let updated = upsert(&pool, user_id, "dark", "#112233", "calendar")
            .await
            .unwrap();
        let loaded = get(&pool, user_id).await.unwrap().unwrap();

        assert_eq!(updated.theme, "dark");
        assert_eq!(updated.accent_color, "#112233");
        assert_eq!(updated.default_view, "calendar");
        assert_eq!(loaded.theme, "dark");
    }
}
