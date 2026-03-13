use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::models::SessionRecord;

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

pub async fn create_session(pool: &PgPool, user_id: Uuid) -> Result<SessionRecord, sqlx::Error> {
    sqlx::query_as::<_, SessionRecord>(
        r#"
        INSERT INTO sessions (user_id, expires_at)
        VALUES ($1, $2)
        RETURNING id, user_id, created_at, expires_at, revoked_at
        "#,
    )
    .bind(user_id)
    .bind(Utc::now() + Duration::days(7))
    .fetch_one(pool)
    .await
}

pub async fn find_active_session(
    pool: &PgPool,
    session_id: Uuid,
    user_id: Uuid,
) -> Result<Option<SessionRecord>, sqlx::Error> {
    sqlx::query_as::<_, SessionRecord>(
        r#"
        SELECT id, user_id, created_at, expires_at, revoked_at
        FROM sessions
        WHERE id = $1
          AND user_id = $2
          AND revoked_at IS NULL
          AND expires_at > NOW()
        "#,
    )
    .bind(session_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn revoke_session(
    pool: &PgPool,
    session_id: Uuid,
    user_id: Uuid,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE sessions
        SET revoked_at = NOW()
        WHERE id = $1
          AND user_id = $2
          AND revoked_at IS NULL
        "#,
    )
    .bind(session_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}
