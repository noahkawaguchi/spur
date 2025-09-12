use crate::{
    domain::{
        RepoError,
        friendship::{FriendshipRepo, FriendshipStatus, user_id_pair::UserIdPair},
        user::UserRepo,
    },
    models::user::{NewUser, User},
};
use sqlx::PgExecutor;

#[allow(clippy::type_complexity)]
#[derive(Default)]
pub struct MockUserRepo {
    pub insert_new: Option<Box<dyn Fn(&NewUser) -> Result<User, RepoError> + Send + Sync>>,
    pub get_by_id: Option<Box<dyn Fn(i32) -> Result<Option<User>, RepoError> + Send + Sync>>,
    pub get_by_email: Option<Box<dyn Fn(&str) -> Result<Option<User>, RepoError> + Send + Sync>>,
    pub get_by_username: Option<Box<dyn Fn(&str) -> Result<Option<User>, RepoError> + Send + Sync>>,
}

#[async_trait::async_trait]
impl UserRepo for MockUserRepo {
    async fn insert_new(
        &self,
        _exec: impl PgExecutor<'_>,
        new_user: &NewUser,
    ) -> Result<User, RepoError> {
        (self.insert_new.as_ref().unwrap())(new_user)
    }
    async fn get_by_id(
        &self,
        _exec: impl PgExecutor<'_>,
        id: i32,
    ) -> Result<Option<User>, RepoError> {
        (self.get_by_id.as_ref().unwrap())(id)
    }
    async fn get_by_email(
        &self,
        _exec: impl PgExecutor<'_>,
        email: &str,
    ) -> Result<Option<User>, RepoError> {
        (self.get_by_email.as_ref().unwrap())(email)
    }
    async fn get_by_username(
        &self,
        _exec: impl PgExecutor<'_>,
        username: &str,
    ) -> Result<Option<User>, RepoError> {
        (self.get_by_username.as_ref().unwrap())(username)
    }
}

#[allow(clippy::type_complexity)]
#[derive(Default)]
pub struct MockFriendshipRepo {
    pub new_request: Option<Box<dyn Fn(&UserIdPair, i32) -> Result<(), RepoError> + Send + Sync>>,
    pub accept_request: Option<Box<dyn Fn(&UserIdPair) -> Result<(), RepoError> + Send + Sync>>,
    pub get_status:
        Option<Box<dyn Fn(&UserIdPair) -> Result<FriendshipStatus, RepoError> + Send + Sync>>,
}

#[async_trait::async_trait]
impl FriendshipRepo for MockFriendshipRepo {
    async fn new_request(
        &self,
        _exec: impl PgExecutor<'_>,
        ids: &UserIdPair,
        requester_id: i32,
    ) -> Result<(), RepoError> {
        (self.new_request.as_ref().unwrap())(ids, requester_id)
    }

    async fn accept_request(
        &self,
        _exec: impl PgExecutor<'_>,
        ids: &UserIdPair,
    ) -> Result<(), RepoError> {
        (self.accept_request.as_ref().unwrap())(ids)
    }

    async fn get_status(
        &self,
        _exec: impl PgExecutor<'_>,
        ids: &UserIdPair,
    ) -> Result<FriendshipStatus, RepoError> {
        (self.get_status.as_ref().unwrap())(ids)
    }
}
