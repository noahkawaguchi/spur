use crate::{
    domain::{RepoError, user::error::UserError},
    models::user::{NewUser, User},
};
use sqlx::PgExecutor;

pub mod error;
pub mod service;

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait UserSvc: Send + Sync {
    async fn insert_new(&self, new_user: &NewUser) -> Result<User, UserError>;
    async fn get_by_id(&self, id: i32) -> Result<User, UserError>;
    async fn get_by_email(&self, email: &str) -> Result<User, UserError>;
    async fn get_by_username(&self, username: &str) -> Result<User, UserError>;
}

#[async_trait::async_trait]
pub trait UserRepo: Send + Sync {
    async fn insert_new(
        &self,
        exec: impl PgExecutor<'_>,
        new_user: &NewUser,
    ) -> Result<User, RepoError>;

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

    async fn get_by_username(
        &self,
        exec: impl PgExecutor<'_>,
        username: &str,
    ) -> Result<Option<User>, RepoError>;
}
