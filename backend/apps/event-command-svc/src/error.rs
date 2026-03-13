#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    Config(String),
    #[error("{0}")]
    Internal(String),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

pub fn is_constraint(error: &sqlx::Error, constraint_name: &str) -> bool {
    match error {
        sqlx::Error::Database(database_error) => {
            database_error.constraint() == Some(constraint_name)
        }
        _ => false,
    }
}
