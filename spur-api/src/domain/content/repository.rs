use crate::{
    models::{post::Post, prompt::Prompt},
    repository::insertion_error::InsertionError,
    technical_error::TechnicalError,
};
use spur_shared::models::{PostWithPrompt, PromptWithAuthor};

#[async_trait::async_trait]
pub trait PromptStore: Send + Sync {
    /// Attempts to insert a new prompt into the database, returning the ID of the newly created
    /// prompt.
    async fn insert_new(&self, author_id: i32, body: &str) -> Result<i32, InsertionError>;

    /// Retrieves a prompt from the database by its ID, returning None if no prompt is found.
    async fn get_by_id(&self, id: i32) -> Result<Option<Prompt>, TechnicalError>;

    /// Retrieves all prompts written by a specific user.
    async fn single_user_prompts(
        &self,
        user_id: i32,
    ) -> Result<Vec<PromptWithAuthor>, TechnicalError>;

    /// Retrieves all prompts written by friends of a specific user
    async fn all_friend_prompts(
        &self,
        user_id: i32,
    ) -> Result<Vec<PromptWithAuthor>, TechnicalError>;
}

#[async_trait::async_trait]
pub trait PostStore: Send + Sync {
    /// Attempts to insert a new post into the database, returning the ID of the newly created
    /// post.
    async fn insert_new(
        &self,
        author_id: i32,
        prompt_id: i32,
        body: &str,
    ) -> Result<i32, InsertionError>;

    /// Retrieves a post from the database by its ID, returning None if no post is found.
    async fn get_by_id(&self, id: i32) -> Result<Option<Post>, TechnicalError>;

    /// Retrieves all posts written by a specific user.
    async fn single_user_posts(
        &self,
        author_id: i32,
    ) -> Result<Vec<PostWithPrompt>, TechnicalError>;

    /// Retrieves all posts written by friends of a specific user
    async fn all_friend_posts(&self, user_id: i32) -> Result<Vec<PostWithPrompt>, TechnicalError>;
}
