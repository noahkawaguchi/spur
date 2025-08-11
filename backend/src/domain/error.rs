use super::{
    auth::AuthError,
    friendship::error::FriendshipError,
    post::{PostError, PostInsertionError},
    user::{UserError, UserInsertionError},
};
use crate::{repository::insertion_error::InsertionError, technical_error::TechnicalError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error(transparent)]
    Auth(#[from] AuthError),

    #[error(transparent)]
    User(#[from] UserError),

    #[error(transparent)]
    Friendship(#[from] FriendshipError),

    #[error(transparent)]
    Post(#[from] PostError),

    #[error(transparent)]
    Technical(#[from] TechnicalError),
}

impl From<PostInsertionError> for DomainError {
    fn from(err: PostInsertionError) -> Self {
        match err {
            PostInsertionError::Database(db_err) => match db_err {
                InsertionError::UniqueViolation(_) => PostError::DuplicateReply.into(),
                InsertionError::Technical(e) => TechnicalError::Database(e).into(),
            },
            PostInsertionError::UnexpectedStatus => {
                TechnicalError::Unexpected(err.to_string()).into()
            }
            PostInsertionError::NotFound => PostError::NotFound.into(),
            PostInsertionError::DeletedParent => PostError::DeletedParent.into(),
            PostInsertionError::ArchivedParent => PostError::ArchivedParent.into(),
            PostInsertionError::SelfReply => PostError::SelfReply.into(),
        }
    }
}

impl From<UserInsertionError> for DomainError {
    fn from(err: UserInsertionError) -> Self {
        match err.0 {
            InsertionError::UniqueViolation(v) if v.contains("email") => {
                UserError::DuplicateEmail.into()
            }
            InsertionError::UniqueViolation(v) if v.contains("username") => {
                UserError::DuplicateUsername.into()
            }
            InsertionError::UniqueViolation(v) => {
                TechnicalError::Unexpected(format!("Unexpected unique violation: {v}")).into()
            }
            InsertionError::Technical(e) => TechnicalError::Database(e).into(),
        }
    }
}
