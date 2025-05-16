use crate::repositories::{friendship_repo::FriendshipStatus, user_repo::UserStore};
use std::sync::Arc;

pub enum FriendshipOutcome {
    AlreadyFriends,
    BecameFriends,
    AlreadyRequested,
    CreatedRequest,
}

pub trait FriendshipStore: Send + Sync {
    async fn new_request(
        &self,
        first_id: i32,
        second_id: i32,
        requester_id: i32,
    ) -> sqlx::Result<()>;

    async fn accept_request(&self, first_id: i32, second_id: i32) -> sqlx::Result<()>;

    async fn get_status(&self, first_id: i32, second_id: i32) -> sqlx::Result<FriendshipStatus>;

    async fn get_friends(&self, id: i32) -> sqlx::Result<Vec<i32>>;

    async fn get_requests(&self, id: i32) -> sqlx::Result<Vec<i32>>;
}

struct FriendshipSvc<S: FriendshipStore> {
    friendship_store: S,
    user_store: Arc<dyn UserStore>,
}

impl<S: FriendshipStore> FriendshipSvc<S> {
    async fn add_friend(
        &self,
        sender_id: i32,
        recipient_username: &str,
    ) -> sqlx::Result<FriendshipOutcome> {
        // First find the recipient's ID
        let recipient_id = self
            .user_store
            .get_by_username(recipient_username)
            .await?
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
            // Already friends, no action needed
            FriendshipStatus::Friends => Ok(FriendshipOutcome::AlreadyFriends),
            // A request from this sender to this recipient already exists, no action needed
            FriendshipStatus::PendingFrom(id) if id == sender_id => {
                Ok(FriendshipOutcome::AlreadyRequested)
            }
            // There is already a pending request in the opposite direction,
            // so accept the existing request
            FriendshipStatus::PendingFrom(_) => {
                self.friendship_store
                    .accept_request(first_id, second_id)
                    .await?;
                Ok(FriendshipOutcome::BecameFriends)
            }
            // No existing relationship, create a new request
            FriendshipStatus::Nil => {
                self.friendship_store
                    .new_request(first_id, second_id, sender_id)
                    .await?;
                Ok(FriendshipOutcome::CreatedRequest)
            }
        }
    }

    async fn get_friends(&self, id: i32) -> sqlx::Result<Vec<String>> {
        futures::future::try_join_all(
            self.friendship_store
                .get_friends(id)
                .await?
                .into_iter()
                .map(|id| async move { Ok(self.user_store.get_by_id(id).await?.username) }),
        )
        .await
    }

    async fn get_requests(&self, id: i32) -> sqlx::Result<Vec<String>> {
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
