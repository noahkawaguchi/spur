use crate::{domain::user::UserError, repository::error::RepoError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FriendshipError {
    #[error("Specified user does not exist")]
    NonexistentUser,

    #[error("Impossible to have a friendship with oneself")]
    SelfFriendship,

    #[error("Already friends with this user")]
    AlreadyFriends,

    #[error("Pending friend request to this user already exists")]
    AlreadyRequested,

    // TODO: this variant should likely be made unnecessary, ideally by ending the dependency on
    // the user service
    #[error(transparent)]
    User(#[from] UserError),

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl From<RepoError> for FriendshipError {
    fn from(e: RepoError) -> Self {
        // TODO: the friendship domain needs to be redesigned
        Self::Internal(e.into())
    }
}
