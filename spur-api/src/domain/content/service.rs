use crate::domain::error::DomainError;
use spur_shared::models::{PostWithPrompt, PromptWithAuthor};

#[async_trait::async_trait]
pub trait PromptManager: Send + Sync {
    /// Attempts to create a new prompt, returning it as a `PromptWithAuthor` if successful.
    async fn create_new(&self, author_id: i32, body: &str)
    -> Result<PromptWithAuthor, DomainError>;

    /// Retrieves a prompt to be used in writing a post. The requester (potential post author) must
    /// be friends with and distinct from the author of the prompt.
    async fn get_for_writing(
        &self,
        requester_id: i32,
        prompt_id: i32,
    ) -> Result<PromptWithAuthor, DomainError>;

    /// Retrieves all prompts written by a specific user.
    async fn single_user_prompts(&self, user_id: i32)
    -> Result<Vec<PromptWithAuthor>, DomainError>;

    /// Retrieves all prompts written by friends of a specific user.
    async fn all_friend_prompts(&self, user_id: i32) -> Result<Vec<PromptWithAuthor>, DomainError>;
}

#[async_trait::async_trait]
pub trait PostManager: Send + Sync {
    /// Attempts to create a new post, returning it as a `PostWithPrompt` if successful.
    async fn create_new(
        &self,
        author_id: i32,
        prompt_id: i32,
        body: &str,
    ) -> Result<PostWithPrompt, DomainError>;

    /// Retrieves a post to be read. The requester (potential reader of the post) and the post
    /// author must be either friends or the same user. The requester does not need to be friends
    /// with the author of the prompt that the post was written in response to.
    async fn get_for_reading(
        &self,
        requester_id: i32,
        post_id: i32,
    ) -> Result<PostWithPrompt, DomainError>;

    /// Retrieves all posts written by a specific user.
    async fn single_user_posts(&self, author_id: i32) -> Result<Vec<PostWithPrompt>, DomainError>;

    /// Retrieves all posts written by friends of a specific user.
    async fn all_friend_posts(&self, user_id: i32) -> Result<Vec<PostWithPrompt>, DomainError>;
}

#[async_trait::async_trait]
pub trait ContentManager: Send + Sync {
    /// Retrieves all prompts and posts written by the requester.
    async fn own_content(
        &self,
        user_id: i32,
    ) -> Result<(Vec<PromptWithAuthor>, Vec<PostWithPrompt>), DomainError>;

    /// Retrieves all prompts and posts written by the user with the specified username, who must
    /// be a friend of the requester.
    async fn specific_friend_content(
        &self,
        requester_id: i32,
        friend_username: &str,
    ) -> Result<(Vec<PromptWithAuthor>, Vec<PostWithPrompt>), DomainError>;

    /// Retrieves all prompts and posts written by friends of the requester.
    async fn all_friend_content(
        &self,
        user_id: i32,
    ) -> Result<(Vec<PromptWithAuthor>, Vec<PostWithPrompt>), DomainError>;
}
