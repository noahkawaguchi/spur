use crate::domain::error::DomainError;
use spur_shared::models::PromptWithAuthor;

#[async_trait::async_trait]
pub trait ContentManager: Send + Sync {
    async fn new_prompt(&self, author_id: i32, body: &str)
    -> Result<PromptWithAuthor, DomainError>;

    async fn get_prompt_for_writing(
        &self,
        requester_id: i32,
        prompt_id: i32,
    ) -> Result<PromptWithAuthor, DomainError>;

    async fn own_prompts(&self, user_id: i32) -> Result<Vec<PromptWithAuthor>, DomainError>;

    async fn specific_friend_prompts(
        &self,
        requester_id: i32,
        friend_username: &str,
    ) -> Result<Vec<PromptWithAuthor>, DomainError>;

    async fn all_friend_prompts(&self, user_id: i32) -> Result<Vec<PromptWithAuthor>, DomainError>;
}
