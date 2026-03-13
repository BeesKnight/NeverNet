use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_default_settings(pool: &PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO ui_settings (user_id, theme)
        VALUES ($1, 'system')
        ON CONFLICT (user_id) DO NOTHING
        "#,
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}
