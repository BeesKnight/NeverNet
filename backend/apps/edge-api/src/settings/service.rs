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
const ALLOWED_DEFAULT_VIEWS: [&str; 4] = ["dashboard", "events", "calendar", "reports"];

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
    let current = repository::get(&state.pool, user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("UI settings not found".to_string()))?;

    let theme = match payload.theme {
        Some(theme) => validate_theme(&theme)?,
        None => current.theme,
    };
    let accent_color = match payload.accent_color {
        Some(accent_color) => validate_accent_color(&accent_color)?,
        None => current.accent_color,
    };
    let default_view = match payload.default_view {
        Some(default_view) => validate_default_view(&default_view)?,
        None => current.default_view,
    };

    Ok(repository::upsert(&state.pool, user_id, &theme, &accent_color, &default_view).await?)
}

fn validate_theme(value: &str) -> Result<String, AppError> {
    let theme = value.trim().to_lowercase();

    if !ALLOWED_THEMES.contains(&theme.as_str()) {
        return Err(AppError::BadRequest(
            "Theme must be one of: light, dark, system".to_string(),
        ));
    }

    Ok(theme)
}

fn validate_default_view(value: &str) -> Result<String, AppError> {
    let default_view = value.trim().to_lowercase();

    if !ALLOWED_DEFAULT_VIEWS.contains(&default_view.as_str()) {
        return Err(AppError::BadRequest(
            "Default view must be one of: dashboard, events, calendar, reports".to_string(),
        ));
    }

    Ok(default_view)
}

fn validate_accent_color(value: &str) -> Result<String, AppError> {
    let accent_color = value.trim().to_lowercase();
    let is_hex_color = accent_color.len() == 7
        && accent_color.starts_with('#')
        && accent_color
            .chars()
            .skip(1)
            .all(|character| character.is_ascii_hexdigit());

    if !is_hex_color {
        return Err(AppError::BadRequest(
            "Accent color must be a hex value like #b6532f".to_string(),
        ));
    }

    Ok(accent_color)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_accent_color() {
        assert!(validate_accent_color("orange").is_err());
    }

    #[test]
    fn accepts_supported_default_view() {
        assert_eq!(validate_default_view("calendar").unwrap(), "calendar");
    }
}
