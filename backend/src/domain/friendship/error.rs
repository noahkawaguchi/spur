use crate::domain::RepoError;
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

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl From<RepoError> for FriendshipError {
    fn from(e: RepoError) -> Self {
        // Other variants are created explicitly
        Self::Internal(e.into())
    }
}
