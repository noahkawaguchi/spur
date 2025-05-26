use super::error::DomainError;
use crate::models::user::{User, UserRegistration};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("An account with the same email already exists")]
    DuplicateEmail,

    #[error("An account with the same username already exists")]
    DuplicateUsername,

    #[error("Invalid email")]
    InvalidEmail,

    #[error("Invalid password")]
    InvalidPassword,
}

#[async_trait::async_trait]
pub trait Authenticator: Send + Sync {
    /// Hashes the password and attempts to create a new user in the database.
    async fn register(&self, reg: UserRegistration) -> Result<(), DomainError>;

    /// Checks `email` and `password` for a valid match in the database.
    async fn validate_credentials(&self, email: &str, password: &str) -> Result<User, DomainError>;
}
