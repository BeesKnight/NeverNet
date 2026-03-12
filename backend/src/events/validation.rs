use chrono::{DateTime, Utc};

use crate::error::AppError;

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
