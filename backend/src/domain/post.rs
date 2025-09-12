use crate::domain::RepoError;
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
    Internal(#[from] anyhow::Error),
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
                Self::Internal(anyhow!("Unexpected unique violation: {v}"))
            }
            RepoError::CheckViolation(v) if v == "text_non_empty" => {
                Self::Internal(anyhow!("Empty field made it past request validation: {v}"))
            }
            RepoError::CheckViolation(v) => {
                Self::Internal(anyhow!("Unexpected check violation: {v}"))
            }
            RepoError::Technical(e) => Self::Internal(e),
        }
    }
}

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait PostManager: Send + Sync {
    /// Attempts to create a new post.
    async fn create_new(&self, author_id: i32, parent_id: i32, body: &str)
    -> Result<(), PostError>;
}

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait PostRepo: Send + Sync {
    /// Attempts to insert a new post into the database.
    async fn insert_new(
        &self,
        author_id: i32,
        parent_id: i32,
        body: &str,
    ) -> Result<PostInsertionOutcome, RepoError>;
}
