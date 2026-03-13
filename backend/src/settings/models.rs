use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct UiSettings {
    pub user_id: Uuid,
    pub theme: String,
    pub accent_color: String,
    pub default_view: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Default, Deserialize)]
pub struct UpdateSettingsRequest {
    pub theme: Option<String>,
    pub accent_color: Option<String>,
    pub default_view: Option<String>,
}
