use crate::domain::error::DomainError;
use spur_shared::models::{PostWithPrompt, PromptWithAuthor};

#[async_trait::async_trait]
pub trait PromptManager: Send + Sync {
    async fn create_new(&self, author_id: i32, body: &str)
    -> Result<PromptWithAuthor, DomainError>;

    async fn get_for_writing(
        &self,
        requester_id: i32,
        prompt_id: i32,
    ) -> Result<PromptWithAuthor, DomainError>;

    async fn single_user_prompts(&self, user_id: i32)
    -> Result<Vec<PromptWithAuthor>, DomainError>;

    async fn all_friend_prompts(&self, user_id: i32) -> Result<Vec<PromptWithAuthor>, DomainError>;
}

#[async_trait::async_trait]
pub trait PostManager: Send + Sync {
    async fn create_new(
        &self,
        author_id: i32,
        prompt_id: i32,
        body: &str,
    ) -> Result<PostWithPrompt, DomainError>;

    async fn get_for_reading(
        &self,
        requester_id: i32,
        post_id: i32,
    ) -> Result<PostWithPrompt, DomainError>;

    async fn single_user_posts(&self, author_id: i32) -> Result<Vec<PostWithPrompt>, DomainError>;

    async fn all_friend_posts(&self, user_id: i32) -> Result<Vec<PostWithPrompt>, DomainError>;
}

#[async_trait::async_trait]
pub trait ContentManager: Send + Sync {
    async fn own_content(
        &self,
        user_id: i32,
    ) -> Result<(Vec<PromptWithAuthor>, Vec<PostWithPrompt>), DomainError>;

    async fn specific_friend_content(
        &self,
        requester_id: i32,
        friend_username: &str,
    ) -> Result<(Vec<PromptWithAuthor>, Vec<PostWithPrompt>), DomainError>;

    async fn all_friend_content(
        &self,
        user_id: i32,
    ) -> Result<(Vec<PromptWithAuthor>, Vec<PostWithPrompt>), DomainError>;
}
