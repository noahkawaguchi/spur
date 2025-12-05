use crate::models::post::PostWithAuthor;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReadError {
    #[error("Not found")]
    NotFound,

    #[error(transparent)]
    Technical(#[from] anyhow::Error),
}

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait SocialRead: Send + Sync {
    /// Retrieves the usernames of all confirmed friends of the user with the provided ID in
    /// descending order of confirmation time (most recent first).
    async fn friend_usernames(&self, id: i32) -> Result<Vec<String>, ReadError>;

    /// Retrieves the usernames of all users who have pending requests to the user with the
    /// provided ID in descending order of request time (most recent first).
    async fn pending_requests(&self, id: i32) -> Result<Vec<String>, ReadError>;

    /// Retrieves all posts written by friends of a specific user in descending order of creation
    /// time (most recent first).
    async fn friend_posts(&self, user_id: i32) -> Result<Vec<PostWithAuthor>, ReadError>;
}

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait PostWithAuthorRead: Send + Sync {
    /// Retrieves a post and its author's username by its post ID.
    async fn by_post_id(&self, id: i32) -> Result<PostWithAuthor, ReadError>;

    /// Retrieves all children of the post with the provided ID and the usernames of the authors of
    /// the posts in descending order of creation time (most recent first).
    async fn by_parent(&self, parent_id: i32) -> Result<Vec<PostWithAuthor>, ReadError>;

    /// Retrieves all posts written by the user with the provided ID along with the user's username
    /// in descending order of creation time (most recent first).
    async fn by_author(&self, author_id: i32) -> Result<Vec<PostWithAuthor>, ReadError>;

    /// Retrieves all posts written by the user with the provided username in descending order of
    /// creation time (most recent first).
    async fn by_author_username(
        &self,
        author_username: &str,
    ) -> Result<Vec<PostWithAuthor>, ReadError>;
}
