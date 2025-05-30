use crate::{
    models::{post::Post, prompt::Prompt},
    repository::insertion_error::InsertionError,
    technical_error::TechnicalError,
};
use spur_shared::models::PromptWithAuthor;

#[async_trait::async_trait]
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

#[async_trait::async_trait]
pub trait PostStore: Send + Sync {
    async fn insert_new(
        &self,
        author_id: i32,
        prompt_id: i32,
        body: &str,
    ) -> Result<i32, InsertionError>;

    async fn get_by_id(&self, id: i32) -> Result<Option<Post>, TechnicalError>;

    async fn get_user_posts(&self, author_id: i32) -> Result<Vec<Post>, TechnicalError>;

    async fn get_friend_posts(&self, user_id: i32) -> Result<Vec<Post>, TechnicalError>;
}
