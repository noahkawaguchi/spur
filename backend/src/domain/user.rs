use crate::{
    models::user::{NewUser, User},
    repository::error::RepoError,
};
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
    Technical(#[from] anyhow::Error),
}

impl From<RepoError> for UserError {
    fn from(e: RepoError) -> Self {
        match e {
            RepoError::UniqueViolation(v) if v == "users_email_unique" => Self::DuplicateEmail,
            RepoError::UniqueViolation(v) if v == "users_username_unique" => {
                Self::DuplicateUsername
            }
            RepoError::UniqueViolation(v) => {
                Self::Technical(anyhow!("Unexpected unique violation: {v}"))
            }
            RepoError::Technical(e) => Self::Technical(e.into()),
            RepoError::Unexpected(e) => Self::Technical(e),
        }
    }
}

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait UserManager: Send + Sync {
    async fn insert_new(&self, new_user: &NewUser) -> Result<User, UserError>;
    async fn get_by_id(&self, id: i32) -> Result<User, UserError>;
    async fn get_by_email(&self, email: &str) -> Result<User, UserError>;
    async fn get_by_username(&self, username: &str) -> Result<User, UserError>;
}

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait UserStore: Send + Sync {
    async fn insert_new(&self, new_user: &NewUser) -> Result<User, RepoError>;
    async fn get_by_id(&self, id: i32) -> Result<Option<User>, RepoError>;
    async fn get_by_email(&self, email: &str) -> Result<Option<User>, RepoError>;
    async fn get_by_username(&self, username: &str) -> Result<Option<User>, RepoError>;
}
