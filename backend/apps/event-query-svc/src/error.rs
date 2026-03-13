#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Config(String),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}
