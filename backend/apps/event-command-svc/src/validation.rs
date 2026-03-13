use chrono::{DateTime, Utc};

use crate::error::AppError;

pub fn validate_category(name: &str, color: &str) -> Result<(), AppError> {
    if name.trim().len() < 2 {
        return Err(AppError::BadRequest(
            "Category name must be at least 2 characters long".to_string(),
        ));
    }

    let value = color.trim();
    if value.len() != 7 || !value.starts_with('#') {
        return Err(AppError::BadRequest(
            "Color must be a hex value like #0F766E".to_string(),
        ));
    }

    Ok(())
}

pub fn validate_event(
    title: &str,
    location: &str,
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
    budget: f64,
) -> Result<(), AppError> {
    if title.trim().len() < 3 {
        return Err(AppError::BadRequest(
            "Event title must be at least 3 characters long".to_string(),
        ));
    }

    if location.trim().len() < 2 {
        return Err(AppError::BadRequest(
            "Location must be at least 2 characters long".to_string(),
        ));
    }

    if ends_at <= starts_at {
        return Err(AppError::BadRequest(
            "Event end time must be after the start time".to_string(),
        ));
    }

    if budget < 0.0 {
        return Err(AppError::BadRequest(
            "Budget cannot be negative".to_string(),
        ));
    }

    Ok(())
}

pub fn validate_status(value: &str) -> Result<(), AppError> {
    if matches!(value, "planned" | "in_progress" | "completed" | "cancelled") {
        Ok(())
    } else {
        Err(AppError::BadRequest(
            "Status must be planned, in_progress, completed, or cancelled".to_string(),
        ))
    }
}

pub fn validate_transition(current: &str, next: &str) -> Result<(), AppError> {
    if current == next {
        return Ok(());
    }

    let valid = matches!(
        (current, next),
        ("planned", "in_progress")
            | ("planned", "cancelled")
            | ("in_progress", "completed")
            | ("in_progress", "cancelled")
    );

    if valid {
        Ok(())
    } else {
        Err(AppError::BadRequest(format!(
            "Invalid status transition from {current} to {next}"
        )))
    }
}
