use crate::{
    error::{ApiError, TechnicalError},
    handlers::friendship_handlers::FriendshipManager,
    repositories::{friendship_repo::FriendshipStatus, user_repo::UserStore},
};
use std::sync::Arc;

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

pub struct FriendshipSvc<S: FriendshipStore> {
    friendship_store: S,
    user_store: Arc<dyn UserStore>,
}

impl<S: FriendshipStore> FriendshipSvc<S> {
    pub fn new(friendship_store: S, user_store: Arc<dyn UserStore>) -> Self {
        Self { friendship_store, user_store }
    }
}

#[async_trait::async_trait]
impl<S: FriendshipStore> FriendshipManager for FriendshipSvc<S> {
    async fn add_friend(&self, sender_id: i32, recipient_username: &str) -> Result<bool, ApiError> {
        // First find the recipient's ID
        let recipient_id = self
            .user_store
            .get_by_username(recipient_username)
            .await?
            .ok_or_else(|| {
                ApiError::Nonexistent(format!(
                    "There is no user with the username {recipient_username}"
                ))
            })?
            .id;

        // Determine how this pair would be stored in the database
        let (first_id, second_id) = if sender_id < recipient_id {
            (sender_id, recipient_id)
        } else {
            (recipient_id, sender_id)
        };

        // Get their current status
        let status = self
            .friendship_store
            .get_status(first_id, second_id)
            .await?;

        match status {
            // Already friends, cannot request to become friends
            FriendshipStatus::Friends => Err(ApiError::Duplicate(format!(
                "Already friends with {recipient_username}"
            ))),

            // A request from this sender to this recipient already exists, cannot request again
            FriendshipStatus::PendingFrom(id) if id == sender_id => Err(ApiError::Duplicate(
                format!("A pending friend request to {recipient_username} already exists"),
            )),

            // Already a pending request in the opposite direction, so accept it
            FriendshipStatus::PendingFrom(_) => {
                self.friendship_store
                    .accept_request(first_id, second_id)
                    .await?;
                Ok(true)
            }

            // No existing relationship, create a new request
            FriendshipStatus::Nil => {
                self.friendship_store
                    .new_request(first_id, second_id, sender_id)
                    .await?;
                Ok(false)
            }
        }
    }

    async fn get_friends(&self, id: i32) -> Result<Vec<String>, TechnicalError> {
        futures::future::try_join_all(
            self.friendship_store
                .get_friends(id)
                .await?
                .into_iter()
                .map(|id| async move { Ok(self.user_store.get_by_id(id).await?.username) }),
        )
        .await
    }

    async fn get_requests(&self, id: i32) -> Result<Vec<String>, TechnicalError> {
        futures::future::try_join_all(
            self.friendship_store
                .get_requests(id)
                .await?
                .into_iter()
                .map(|id| async move { Ok(self.user_store.get_by_id(id).await?.username) }),
        )
        .await
    }
}
