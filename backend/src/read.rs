use crate::models::post::PostWithAuthor;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReadError {
    #[error("{0}")]
    NotFound(String),
    #[error(transparent)]
    Technical(#[from] anyhow::Error),
}

#[async_trait::async_trait]
pub trait SocialRead: Send + Sync {
    /// Retrieves the usernames of all confirmed friends of the user with the provided ID.
    async fn friend_usernames(&self, id: i32) -> Result<Vec<String>, ReadError>;
    /// Retrieves the usernames of all users who have pending requests to the user with the
    /// provided ID.
    async fn pending_requests(&self, id: i32) -> Result<Vec<String>, ReadError>;
    /// Retrieves all posts written by friends of a specific user.
    async fn all_friend_posts(&self, user_id: i32) -> Result<Vec<PostWithAuthor>, ReadError>;
}

#[async_trait::async_trait]
pub trait PostWithAuthorRead: Send + Sync {
    /// Retrieves a post and its author's username by its post ID.
    async fn get_by_id(&self, id: i32) -> Result<PostWithAuthor, ReadError>;
    /// Retrieves all children of the post with the provided ID and the usernames of the authors of
    /// the posts.
    async fn get_by_parent_id(&self, parent_id: i32) -> Result<Vec<PostWithAuthor>, ReadError>;
    /// Retrieves all posts written by the user with the provided ID along with the user's username.
    async fn user_posts_by_id(&self, author_id: i32) -> Result<Vec<PostWithAuthor>, ReadError>;
    /// Retrieves all posts written by the user with the provided username.
    async fn user_posts_by_username(
        &self,
        author_username: &str,
    ) -> Result<Vec<PostWithAuthor>, ReadError>;
}
