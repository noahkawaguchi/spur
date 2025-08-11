use crate::{
    models::user::{NewUser, User},
    repository::insertion_error::InsertionError,
    technical_error::TechnicalError,
};
use thiserror::Error;

use super::error::DomainError;

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
}

/// A specialized error type for inserting users into the database that wraps [`InsertionError`].
/// Used for trait implementations and separation of concerns.
#[cfg_attr(test, derive(Debug))]
pub struct UserInsertionError(pub InsertionError);

impl From<sqlx::Error> for UserInsertionError {
    fn from(err: sqlx::Error) -> Self { Self(err.into()) }
}

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait UserManager: Send + Sync {
    async fn insert_new(&self, new_user: &NewUser) -> Result<i32, DomainError>;
    async fn get_by_id(&self, id: i32) -> Result<User, DomainError>;
    async fn get_by_email(&self, email: &str) -> Result<User, DomainError>;
    async fn get_by_username(&self, username: &str) -> Result<User, DomainError>;
}

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait UserStore: Send + Sync {
    async fn insert_new(&self, new_user: &NewUser) -> Result<i32, UserInsertionError>;
    async fn get_by_id(&self, id: i32) -> Result<Option<User>, TechnicalError>;
    async fn get_by_email(&self, email: &str) -> Result<Option<User>, TechnicalError>;
    async fn get_by_username(&self, username: &str) -> Result<Option<User>, TechnicalError>;
}
