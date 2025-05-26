use crate::{
    models::prompt::Prompt, repository::insertion_error::InsertionError,
    technical_error::TechnicalError,
};
use spur_shared::models::PromptWithAuthor;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PromptError {
    #[error("You have already created the same prompt")]
    Duplicate,

    #[error("Prompt not found")]
    NotFound,

    #[error("You must be friends to see someone's content")]
    NotFriends,
}

pub trait PromptStore: Send + Sync {
    async fn insert_new(&self, author_id: i32, body: &str) -> Result<i32, InsertionError>;

    async fn get_by_id(&self, id: i32) -> Result<Option<Prompt>, TechnicalError>;

    async fn get_user_prompts(&self, user_id: i32)
    -> Result<Vec<PromptWithAuthor>, TechnicalError>;

    async fn get_friend_prompts(
        &self,
        user_id: i32,
    ) -> Result<Vec<PromptWithAuthor>, TechnicalError>;
}
