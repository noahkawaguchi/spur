use crate::domain::RepoError;
use sqlx::PgExecutor;
use user_id_pair::UserIdPair;

pub mod error;
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

#[async_trait::async_trait]
pub trait FriendshipRepo: Send + Sync {
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
