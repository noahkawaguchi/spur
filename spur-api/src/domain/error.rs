use super::{
    auth::AuthError, content::error::ContentError, friendship::error::FriendshipError,
    user::UserError,
};
use crate::technical_error::TechnicalError;
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
    Content(#[from] ContentError),

    #[error(transparent)]
    Technical(#[from] TechnicalError),
}
