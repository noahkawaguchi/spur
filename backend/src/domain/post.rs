use crate::domain::{RepoError, post::error::PostError};

pub mod error;
pub mod service;

#[cfg_attr(test, derive(Debug))]
pub enum PostInsertionOutcome {
    Inserted,
    ParentMissing,
    ParentDeleted,
    ParentArchived,
    SelfReply,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait PostSvc: Send + Sync {
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
