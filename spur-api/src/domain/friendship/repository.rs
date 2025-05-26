use super::FriendshipStatus;
use crate::technical_error::TechnicalError;

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait FriendshipStore: Send + Sync {
    /// Creates a new friend request between the two users.
    ///
    /// - `first_id` should always be less than `second_id`.
    /// - `requester_id`, equal to either `first_id` or `second_id`, indicates who initiated the
    /// request.
    async fn new_request(
        &self,
        first_id: i32,
        second_id: i32,
        requester_id: i32,
    ) -> Result<(), TechnicalError>;

    /// Accepts a pending friend request that involves the two users, regardless of who initiated
    /// it.
    ///
    /// `first_id` should always be less than `second_id`.
    async fn accept_request(&self, first_id: i32, second_id: i32) -> Result<(), TechnicalError>;

    /// Determines the status of the relationship between the two users.
    ///
    /// `first_id` should always be less than `second_id`.
    ///
    /// See [`FriendshipStatus`] for more information on status meanings.
    async fn get_status(
        &self,
        first_id: i32,
        second_id: i32,
    ) -> Result<FriendshipStatus, TechnicalError>;

    /// Retrieves the IDs of all confirmed friends of the user with the provided ID.
    async fn get_friends(&self, id: i32) -> Result<Vec<i32>, TechnicalError>;

    /// Retrieves the IDs of all users who have pending requests to the user with the provided ID.
    async fn get_requests(&self, id: i32) -> Result<Vec<i32>, TechnicalError>;
}
