use crate::domain::RepoError;
use anyhow::{Result, anyhow};

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Account not found")]
    NonexistentAccount,

    #[error(
        "An account with this email already exists. Try logging in or using a different email."
    )]
    DuplicateEmail,

    #[error("Username taken")]
    DuplicateUsername,

    #[error("Invalid password")]
    InvalidPassword,

    #[error("Expired or invalid token. Try logging in again.")]
    TokenValidation,

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl From<RepoError> for AuthError {
    fn from(e: RepoError) -> Self {
        match e {
            RepoError::UniqueViolation(v) if v == "users_email_unique" => Self::DuplicateEmail,
            RepoError::UniqueViolation(v) if v == "users_username_unique" => {
                Self::DuplicateUsername
            }
            RepoError::UniqueViolation(v) => {
                Self::Internal(anyhow!("Unexpected unique violation: {v}"))
            }
            RepoError::CheckViolation(v) if v == "users_username_chars" => Self::Internal(anyhow!(
                "Invalid username made it past request validation: {v}"
            )),
            RepoError::CheckViolation(v) if v == "text_non_empty" => {
                Self::Internal(anyhow!("Empty field made it past request validation: {v}"))
            }
            RepoError::CheckViolation(v) => {
                Self::Internal(anyhow!("Unexpected check violation: {v}"))
            }
            RepoError::Technical(e) => Self::Internal(e),
        }
    }
}

#[cfg_attr(test, mockall::automock)]
pub trait AuthProvider: Send + Sync {
    /// Converts a plaintext password into a hashed version.
    fn hash_pw(&self, pw: &str) -> Result<String>;
    /// Checks whether the password and hash are a valid match.
    fn is_valid_pw(&self, pw: &str, hash: &str) -> Result<bool>;
    /// Creates a new token with the provided payload.
    fn create_token(&self, payload: i32) -> Result<String>;
    /// Validates the token, returning the contained payload if valid.
    fn validate_token(&self, token: &str) -> Result<i32>;
}
