use crate::{
    domain::{
        error::DomainError,
        friendship::{service::FriendshipManager, user_id_pair::UserIdPair},
        prompt::{PromptError, PromptStore},
        user::UserManager,
    },
    repository::insertion_error::InsertionError,
    technical_error::TechnicalError,
};
use spur_shared::models::PromptWithAuthor;
use std::sync::Arc;

pub struct PromptSvc<S: PromptStore> {
    prompt_store: S,
    friendship_svc: Arc<dyn FriendshipManager>,
    user_svc: Arc<dyn UserManager>,
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

        // Must be friends to see someone's prompt
        if self
            .friendship_svc
            .are_friends(&UserIdPair::new(requester_id, prompt.author_id)?)
            .await?
        {
            Ok(PromptWithAuthor {
                id: prompt.id,
                author_username: self.user_svc.get_by_id(prompt.author_id).await?.username,
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
        // Must be friends to see someone's prompts
        if self
            .friendship_svc
            .are_friends(&UserIdPair::new(requester_id, target_id)?)
            .await?
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
