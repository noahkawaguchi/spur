use crate::{models::post::PostInfo, repository::error::RepoError};
use anyhow::anyhow;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PostError {
    #[error("No post found")]
    NotFound,
    // TODO: implement editing posts
    #[error("Cannot reply multiple times to the same post. Try editing the existing reply.")]
    DuplicateReply,
    #[error("Cannot reply to a deleted post")]
    DeletedParent,
    #[error("Cannot reply to an archived post")]
    ArchivedParent,
    #[error("Cannot reply to one's own post")]
    SelfReply,
    #[error(transparent)]
    Technical(#[from] anyhow::Error),
}

#[cfg_attr(test, derive(Debug))]
pub enum PostInsertionOutcome {
    Inserted,
    ParentMissing,
    ParentDeleted,
    ParentArchived,
    SelfReply,
}

impl TryFrom<PostInsertionOutcome> for () {
    type Error = PostError;
    fn try_from(outcome: PostInsertionOutcome) -> Result<Self, Self::Error> {
        match outcome {
            PostInsertionOutcome::Inserted => Ok(()),
            PostInsertionOutcome::ParentMissing => Err(Self::Error::NotFound),
            PostInsertionOutcome::ParentDeleted => Err(Self::Error::DeletedParent),
            PostInsertionOutcome::ParentArchived => Err(Self::Error::ArchivedParent),
            PostInsertionOutcome::SelfReply => Err(Self::Error::SelfReply),
        }
    }
}

impl From<RepoError> for PostError {
    fn from(e: RepoError) -> Self {
        match e {
            RepoError::UniqueViolation(v) if v == "post_author_parent_unique" => {
                Self::DuplicateReply
            }
            RepoError::UniqueViolation(v) => {
                Self::Technical(anyhow!("Unexpected unique violation: {v}"))
            }
            RepoError::Technical(e) => Self::Technical(e.into()),
            RepoError::Unexpected(e) => Self::Technical(e),
        }
    }
}

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait PostManager: Send + Sync {
    /// Attempts to create a new post.
    async fn create_new(&self, author_id: i32, parent_id: i32, body: &str)
    -> Result<(), PostError>;
    /// Retrieves a post by its ID.
    async fn get_by_id(&self, post_id: i32) -> Result<PostInfo, PostError>;
    /// Retrieves all children of the post with the provided ID.
    async fn get_by_parent_id(&self, parent_id: i32) -> Result<Vec<PostInfo>, PostError>;
    /// Retrieves all posts written by the user with the provided ID.
    async fn user_posts_by_id(&self, author_id: i32) -> Result<Vec<PostInfo>, PostError>;
    /// Retrieves all posts written by the user with the provided username.
    async fn user_posts_by_username(
        &self,
        author_username: &str,
    ) -> Result<Vec<PostInfo>, PostError>;
    /// Retrieves all posts written by friends of a specific user.
    async fn all_friend_posts(&self, user_id: i32) -> Result<Vec<PostInfo>, PostError>;
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
    ) -> Result<PostInsertionOutcome, RepoError>;
    /// Retrieves a post by its ID, returning None if no post is found.
    async fn get_by_id(&self, id: i32) -> Result<Option<PostInfo>, RepoError>;
    /// Retrieves all children of the post with the provided ID.
    async fn get_by_parent_id(&self, parent_id: i32) -> Result<Vec<PostInfo>, RepoError>;
    /// Retrieves all posts written by the user with the provided ID.
    async fn user_posts_by_id(&self, author_id: i32) -> Result<Vec<PostInfo>, RepoError>;
    /// Retrieves all posts written by the user with the provided username.
    async fn user_posts_by_username(
        &self,
        author_username: &str,
    ) -> Result<Vec<PostInfo>, RepoError>;
    /// Retrieves all posts written by friends of a specific user.
    async fn all_friend_posts(&self, user_id: i32) -> Result<Vec<PostInfo>, RepoError>;
}
