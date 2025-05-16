use crate::handlers::friendship_handlers::FriendshipOutcome;

pub enum FriendshipStatus {
    Friends,
    PendingFrom(i32),
    Nil,
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
    store: S,
}

impl<S: FriendshipStore> FriendshipSvc<S> {
    async fn add_friend(
        &self,
        sender_id: i32,
        recipient_id: i32,
    ) -> sqlx::Result<FriendshipOutcome> {
        // Determine how this pair would be stored in the database
        let (first_id, second_id) = if sender_id < recipient_id {
            (sender_id, recipient_id)
        } else {
            (recipient_id, sender_id)
        };

        // Get their current status
        let status = self.store.get_status(first_id, second_id).await?;

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
                self.store.accept_request(first_id, second_id).await?;
                Ok(FriendshipOutcome::BecameFriends)
            }
            // No existing relationship, create a new request
            FriendshipStatus::Nil => {
                self.store
                    .new_request(first_id, second_id, sender_id)
                    .await?;
                Ok(FriendshipOutcome::CreatedRequest)
            }
        }
    }
}
