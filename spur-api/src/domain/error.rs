use super::{
    auth::AuthError, friendship::error::FriendshipError, prompt::PromptError, user::UserError,
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
    Prompt(#[from] PromptError),

    #[error(transparent)]
    Technical(#[from] TechnicalError),
}
