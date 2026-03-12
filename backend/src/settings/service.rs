use uuid::Uuid;

use crate::{
    app_state::AppState,
    error::AppError,
    settings::{
        models::{UiSettings, UpdateSettingsRequest},
        repository,
    },
};

const ALLOWED_THEMES: [&str; 3] = ["light", "dark", "system"];

pub async fn get(state: &AppState, user_id: Uuid) -> Result<UiSettings, AppError> {
    repository::get(&state.pool, user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("UI settings not found".to_string()))
}

pub async fn update(
    state: &AppState,
    user_id: Uuid,
    payload: UpdateSettingsRequest,
) -> Result<UiSettings, AppError> {
    let theme = payload.theme.trim().to_lowercase();

    if !ALLOWED_THEMES.contains(&theme.as_str()) {
        return Err(AppError::BadRequest(
            "Theme must be one of: light, dark, system".to_string(),
        ));
    }

    Ok(repository::upsert(&state.pool, user_id, &theme).await?)
}
