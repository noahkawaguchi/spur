use crate::{
    domain::{RepoError, post::error::PostError},
    models::post::Post,
};
use sqlx::PgExecutor;

pub mod error;
pub mod service;

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait PostSvc: Send + Sync {
    /// Attempts to create a new post.
    async fn create_new(&self, author_id: i32, parent_id: i32, body: &str)
    -> Result<(), PostError>;
}

#[async_trait::async_trait]
pub trait PostRepo: Send + Sync {
    async fn insert_new(
        &self,
        exec: impl PgExecutor<'_>,
        author_id: i32,
        parent_id: i32,
        body: &str,
    ) -> Result<(), RepoError>;

    async fn get_by_id(
        &self,
        exec: impl PgExecutor<'_>,
        id: i32,
    ) -> Result<Option<Post>, RepoError>;
}
