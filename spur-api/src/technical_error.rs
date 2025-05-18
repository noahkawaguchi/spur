use crate::services::jwt_svc::JwtCreationError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TechnicalError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Bcrypt error: {0}")]
    Bcrypt(#[from] bcrypt::BcryptError),

    #[error("JWT creation error: {0}")]
    TokenCreation(#[from] JwtCreationError),

    #[error("Technical error: {0}")]
    Other(#[from] anyhow::Error),
}
