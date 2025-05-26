use crate::{
    domain::{
        error::DomainError,
        friendship::{FriendshipStatus, repository::FriendshipStore},
        prompt::{PromptError, PromptStore},
        user::UserStore,
    },
    repository::insertion_error::InsertionError,
    technical_error::TechnicalError,
};
use spur_shared::models::PromptWithAuthor;
use std::sync::Arc;

pub struct PromptSvc<S: PromptStore> {
    prompt_store: S,
    friendship_store: Arc<dyn FriendshipStore>,
    user_store: Arc<dyn UserStore>,
}

impl<S: PromptStore> PromptSvc<S> {
    async fn create_new(&self, author_id: i32, body: &str) -> Result<i32, DomainError> {
        match self.prompt_store.insert_new(author_id, body).await {
            Ok(id) => Ok(id),
            Err(InsertionError::Technical(e)) => Err(TechnicalError::Database(e).into()),
            Err(InsertionError::UniqueViolation(_)) => Err(PromptError::Duplicate.into()),
        }
    }

    async fn get_with_author(
        &self,
        requester_id: i32,
        prompt_id: i32,
    ) -> Result<PromptWithAuthor, DomainError> {
        let prompt = self
            .prompt_store
            .get_by_id(prompt_id)
            .await?
            .ok_or(PromptError::NotFound)?;

        // Determine how this pair's friendship would be stored in the database
        let (first_id, second_id) = if requester_id < prompt.author_id {
            (requester_id, prompt.author_id)
        } else {
            (prompt.author_id, requester_id)
        };

        // Must be friends to see someone's prompt
        if self
            .friendship_store
            .get_status(first_id, second_id)
            .await?
            == FriendshipStatus::Friends
        {
            Ok(PromptWithAuthor {
                id: prompt.id,
                author_username: self.user_store.get_by_id(prompt.author_id).await?.username,
                body: prompt.body,
            })
        } else {
            Err(PromptError::NotFriends.into())
        }
    }

    async fn get_user_prompts(
        &self,
        requester_id: i32,
        target_id: i32,
    ) -> Result<Vec<PromptWithAuthor>, DomainError> {
        // Determine how this pair's friendship would be stored in the database
        let (first_id, second_id) = if requester_id < target_id {
            (requester_id, target_id)
        } else {
            (target_id, requester_id)
        };

        // Must be friends to see someone's prompts
        if self
            .friendship_store
            .get_status(first_id, second_id)
            .await?
            == FriendshipStatus::Friends
        {
            self.prompt_store
                .get_user_prompts(target_id)
                .await
                .map_err(DomainError::from)
        } else {
            Err(PromptError::NotFriends.into())
        }
    }

    async fn get_friend_prompts(&self, user_id: i32) -> Result<Vec<PromptWithAuthor>, DomainError> {
        self.prompt_store
            .get_friend_prompts(user_id)
            .await
            .map_err(DomainError::from)
    }
}
