use crate::{
    auth::models::{LoginRequest, RegisterRequest},
    error::AppError,
};

pub fn validate_register(payload: &RegisterRequest) -> Result<(), AppError> {
    if payload.full_name.trim().len() < 2 {
        return Err(AppError::BadRequest(
            "Full name must be at least 2 characters long".to_string(),
        ));
    }

    validate_email(&payload.email)?;

    if payload.password.len() < 8 {
        return Err(AppError::BadRequest(
            "Password must be at least 8 characters long".to_string(),
        ));
    }

    Ok(())
}

pub fn validate_login(payload: &LoginRequest) -> Result<(), AppError> {
    validate_email(&payload.email)?;

    if payload.password.is_empty() {
        return Err(AppError::BadRequest("Password is required".to_string()));
    }

    Ok(())
}

fn validate_email(email: &str) -> Result<(), AppError> {
    let normalized = email.trim();

    if normalized.len() < 5 || !normalized.contains('@') {
        return Err(AppError::BadRequest(
            "A valid email is required".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_short_password() {
        let payload = RegisterRequest {
            email: "demo@example.com".to_string(),
            password: "short".to_string(),
            full_name: "Demo".to_string(),
        };

        assert!(validate_register(&payload).is_err());
    }
}
