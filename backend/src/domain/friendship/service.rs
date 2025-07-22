use super::user_id_pair::UserIdPair;
use crate::domain::error::DomainError;

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait FriendshipManager: Send + Sync {
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
    async fn add_friend(
        &self,
        sender_id: i32,
        recipient_username: &str,
    ) -> Result<bool, DomainError>;

    /// Retrieves the usernames of all confirmed friends of the user with the provided ID.
    async fn get_friends(&self, id: i32) -> Result<Vec<String>, DomainError>;

    /// Retrieves the usernames of all users who have pending requests to the user with the
    /// provided ID.
    async fn get_requests(&self, id: i32) -> Result<Vec<String>, DomainError>;

    /// Determines whether two users are confirmed friends.
    async fn are_friends(&self, ids: &UserIdPair) -> Result<bool, DomainError>;
}
