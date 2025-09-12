use crate::domain::friendship::error::FriendshipError;

pub mod mutate_friendship_by_username_svc;
pub mod uow;

#[async_trait::async_trait]
pub trait MutateFriendshipByUsername: Send + Sync {
    /// Attempts to add a friendship between the two users, returning whether or not they are now
    /// friends.
    ///
    /// - If there is a pending request from the recipient to the sender (i.e., an existing request
    /// in the opposite direction), the request is accepted and the two users become friends
    /// (returns true).
    /// - If there is no existing relationship, a new request from the sender to the recipient is
    /// created (returns false).
    ///
    /// # Errors
    ///
    /// Will return `Err` if the two users are already friends, or if there is already a pending
    /// request from the sender to the recipient. (In which case nothing is mutated.)
    async fn add_friend_by_username(
        &self,
        sender_id: i32,
        recipient_username: &str,
    ) -> Result<bool, FriendshipError>;
}
