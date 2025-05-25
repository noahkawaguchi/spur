use crate::technical_error::TechnicalError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error(transparent)]
    Auth(#[from] AuthError),

    #[error(transparent)]
    Friendship(#[from] FriendshipError),

    #[error(transparent)]
    Prompt(#[from] PromptError),

    #[error(transparent)]
    Technical(#[from] TechnicalError),
}

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

#[derive(Debug, Error)]
pub enum FriendshipError {
    #[error("The specified user does not exist")]
    NonexistentUser,

    #[error("You are already friends with this user")]
    AlreadyFriends,

    #[error("A pending friend request to this user already exists")]
    AlreadyRequested,
}

#[derive(Debug, Error)]
pub enum PromptError {
    #[error("You have already created the same prompt")]
    Duplicate,

    #[error("Prompt not found")]
    NotFound,

    #[error("You must be friends to see someone's content")]
    NotFriends,
}
