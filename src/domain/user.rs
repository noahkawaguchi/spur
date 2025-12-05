use crate::{
    domain::RepoError,
    models::user::{NewUser, User},
};
use sqlx::PgExecutor;

#[async_trait::async_trait]
pub trait UserRepo: Send + Sync {
    async fn insert_new(
        &self,
        exec: impl PgExecutor<'_>,
        new_user: &NewUser,
    ) -> Result<User, RepoError>;

    #[allow(dead_code)] // Because this basic functionality will likely be necessary in the future
    async fn get_by_id(
        &self,
        exec: impl PgExecutor<'_>,
        id: i32,
    ) -> Result<Option<User>, RepoError>;

    async fn get_by_email(
        &self,
        exec: impl PgExecutor<'_>,
        email: &str,
    ) -> Result<Option<User>, RepoError>;

    /// Fetches a user by username, blocking concurrent writes to the same user until the
    /// surrounding transaction completes.
    async fn get_by_username_exclusive(
        &self,
        exec: impl PgExecutor<'_>,
        username: &str,
    ) -> Result<Option<User>, RepoError>;
}
