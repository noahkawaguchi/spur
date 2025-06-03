use crate::{
    models::{post::Post, prompt::Prompt},
    repository::insertion_error::InsertionError,
    technical_error::TechnicalError,
};
use spur_shared::models::{PostWithPrompt, PromptWithAuthor};

#[async_trait::async_trait]
pub trait PromptStore: Send + Sync {
    async fn insert_new(&self, author_id: i32, body: &str) -> Result<i32, InsertionError>;

    async fn get_by_id(&self, id: i32) -> Result<Option<Prompt>, TechnicalError>;

    async fn single_user_prompts(
        &self,
        user_id: i32,
    ) -> Result<Vec<PromptWithAuthor>, TechnicalError>;

    async fn all_friend_prompts(
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

    async fn single_user_posts(
        &self,
        author_id: i32,
    ) -> Result<Vec<PostWithPrompt>, TechnicalError>;

    async fn all_friend_posts(&self, user_id: i32) -> Result<Vec<PostWithPrompt>, TechnicalError>;
}
