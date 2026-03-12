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
