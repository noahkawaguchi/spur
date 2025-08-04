#[derive(Debug, thiserror::Error)]
pub enum TechnicalError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Bcrypt error: {0}")]
    Bcrypt(#[from] bcrypt::BcryptError),

    #[error("JWT creation error: {0}")]
    JwtCreation(#[from] jsonwebtoken::errors::Error),

    #[error("Unexpected pre-1970 system time: {0}")]
    Pre1970(chrono::DateTime<chrono::Utc>),

    #[error("Unexpected error: {0}")]
    Unexpected(String),
}
