use super::{FriendshipStatus, user_id_pair::UserIdPair};
use crate::repository::error::RepoError;
use sqlx::PgExecutor;

#[async_trait::async_trait]
pub trait FriendshipStore: Send + Sync {
    /// Creates a new friend request between the two users. `requester_id` must be equal to one of
    /// the IDs in the pair, indicating who initiated the request.
    async fn new_request(
        &self,
        exec: impl PgExecutor<'_>,
        ids: &UserIdPair,
        requester_id: i32,
    ) -> Result<(), RepoError>;

    /// Accepts a pending friend request that involves the two users, regardless of who initiated
    /// it.
    async fn accept_request(
        &self,
        exec: impl PgExecutor<'_>,
        ids: &UserIdPair,
    ) -> Result<(), RepoError>;

    /// Determines the status of the relationship between the two users.
    /// See [`FriendshipStatus`] for more information on status meanings.
    async fn get_status(
        &self,
        exec: impl PgExecutor<'_>,
        ids: &UserIdPair,
    ) -> Result<FriendshipStatus, RepoError>;
}
