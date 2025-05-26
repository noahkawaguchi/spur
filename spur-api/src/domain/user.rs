use crate::{
    models::user::{NewUser, User},
    repository::insertion_error::InsertionError,
    technical_error::TechnicalError,
};

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait UserStore: Send + Sync {
    async fn insert_new(&self, new_user: &NewUser) -> Result<(), InsertionError>;
    async fn get_by_id(&self, id: i32) -> Result<User, TechnicalError>;
    async fn get_by_email(&self, email: &str) -> Result<Option<User>, TechnicalError>;
    async fn get_by_username(&self, username: &str) -> Result<Option<User>, TechnicalError>;
}
