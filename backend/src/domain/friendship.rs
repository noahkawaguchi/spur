pub mod error;
pub mod repository;
pub mod service;
pub mod user_id_pair;

#[derive(Debug, PartialEq, Eq)]
pub enum FriendshipStatus {
    /// The two users are confirmed friends.
    Friends,
    /// There is a pending request from the user with the contained ID.
    PendingFrom(i32),
    /// There is no existing relationship between the two users.
    Nil,
}
