use thiserror::Error;

#[derive(Debug, Error)]
pub enum FriendshipError {
    #[error("The specified user does not exist")]
    NonexistentUser,

    #[error("You are already friends with this user")]
    AlreadyFriends,

    #[error("A pending friend request to this user already exists")]
    AlreadyRequested,
}
