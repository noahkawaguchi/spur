use crate::domain::RepoError;
use anyhow::anyhow;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UserError {
    #[error("User account not found")]
    NotFound,

    #[error(
        "An account with this email already exists. Try logging in or using a different email."
    )]
    DuplicateEmail,

    #[error("Username taken")]
    DuplicateUsername,

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl From<RepoError> for UserError {
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
