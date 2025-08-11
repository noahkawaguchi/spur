use crate::{
    domain::error::DomainError, models::post::PostInfo,
    repository::insertion_error::InsertionError, technical_error::TechnicalError,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PostError {
    // TODO: implement editing posts
    #[error("Cannot reply multiple times to the same post. Try editing the existing reply.")]
    DuplicateReply,

    #[error("No post found")]
    NotFound,

    #[error("Cannot reply to a deleted post")]
    DeletedParent,

    #[error("Cannot reply to an archived post")]
    ArchivedParent,

    #[error("Cannot reply to one's own post")]
    SelfReply,
}

/// A specialized error enum for inserting posts into the database.
#[derive(Debug, Error)]
pub enum PostInsertionError {
    /// A technical database error or unique violation enforced by the schema.
    #[error(transparent)]
    Database(#[from] InsertionError),

    /// Should be impossible, as all statuses are defined as hardcoded strings in the SQL query.
    /// This variant is only present for exhaustive matching and future-proofing.
    #[error("Unexpected status returned from post insertion query")]
    UnexpectedStatus,

    #[error("No post found")]
    NotFound,

    #[error("Cannot reply to a deleted post")]
    DeletedParent,

    #[error("Cannot reply to an archived post")]
    ArchivedParent,

    #[error("Cannot reply to one's own post")]
    SelfReply,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait PostManager: Send + Sync {
    /// Attempts to create a new post.
    async fn create_new(
        &self,
        author_id: i32,
        prompt_id: i32,
        body: &str,
    ) -> Result<(), DomainError>;

    /// Retrieves a post by its ID.
    async fn get_by_id(&self, post_id: i32) -> Result<PostInfo, DomainError>;

    /// Retrieves all children of the post with the provided ID.
    async fn get_by_parent_id(&self, parent_id: i32) -> Result<Vec<PostInfo>, DomainError>;

    /// Retrieves all posts written by the user with the provided ID.
    async fn user_posts_by_id(&self, author_id: i32) -> Result<Vec<PostInfo>, DomainError>;

    /// Retrieves all posts written by the user with the provided username.
    async fn user_posts_by_username(
        &self,
        author_username: &str,
    ) -> Result<Vec<PostInfo>, DomainError>;

    /// Retrieves all posts written by friends of a specific user.
    async fn all_friend_posts(&self, user_id: i32) -> Result<Vec<PostInfo>, DomainError>;
}

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait PostStore: Send + Sync {
    /// Attempts to insert a new post into the database.
    async fn insert_new(
        &self,
        author_id: i32,
        parent_id: i32,
        body: &str,
    ) -> Result<(), PostInsertionError>;

    /// Retrieves a post by its ID, returning None if no post is found.
    async fn get_by_id(&self, id: i32) -> Result<Option<PostInfo>, TechnicalError>;

    /// Retrieves all children of the post with the provided ID.
    async fn get_by_parent_id(&self, parent_id: i32) -> Result<Vec<PostInfo>, TechnicalError>;

    /// Retrieves all posts written by the user with the provided ID.
    async fn user_posts_by_id(&self, author_id: i32) -> Result<Vec<PostInfo>, TechnicalError>;

    /// Retrieves all posts written by the user with the provided username.
    async fn user_posts_by_username(
        &self,
        author_username: &str,
    ) -> Result<Vec<PostInfo>, TechnicalError>;

    /// Retrieves all posts written by friends of a specific user.
    async fn all_friend_posts(&self, user_id: i32) -> Result<Vec<PostInfo>, TechnicalError>;
}
