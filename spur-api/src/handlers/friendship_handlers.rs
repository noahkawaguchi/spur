use crate::services::friendship_svc::FriendshipOutcome;

pub trait FriendshipManager {
    /// Attempts to add a friendship between the two users. If the they are already friends, or if
    /// there is a pending request from the sender to the recipient, nothing is changed. If there
    /// is a pending request from the recipient to the sender, the request is accepted and the two
    /// users become friends. If there is no existing relationship, a new request from the sender
    /// to the recipient is created.
    async fn add_friend(
        &self,
        sender_id: i32,
        recipient_username: &str,
    ) -> sqlx::Result<FriendshipOutcome>;

    /// Retrieves the usernames of all confirmed friends of the user with the provided ID.
    async fn get_friends(&self, id: i32) -> sqlx::Result<Vec<String>>;

    /// Retrieves the usernames of all users who have pending requests to the user with the
    /// provided ID.
    async fn get_requests(&self, id: i32) -> sqlx::Result<Vec<String>>;
}
