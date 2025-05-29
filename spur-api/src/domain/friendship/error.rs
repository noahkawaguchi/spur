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
}
