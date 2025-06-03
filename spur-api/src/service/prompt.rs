use crate::{
    domain::{
        content::{error::ContentError, repository::PromptStore, service::PromptManager},
        error::DomainError,
        friendship::{service::FriendshipManager, user_id_pair::UserIdPair},
        user::UserManager,
    },
    repository::insertion_error::InsertionError,
    technical_error::TechnicalError,
};
use spur_shared::models::PromptWithAuthor;
use std::sync::Arc;

pub struct PromptSvc<S: PromptStore> {
    store: S,
    user_svc: Arc<dyn UserManager>,
    friendship_svc: Arc<dyn FriendshipManager>,
}

impl<S: PromptStore> PromptSvc<S> {
    pub const fn new(
        store: S,
        user_svc: Arc<dyn UserManager>,
        friendship_svc: Arc<dyn FriendshipManager>,
    ) -> Self {
        Self { store, user_svc, friendship_svc }
    }
}

#[async_trait::async_trait]
impl<S: PromptStore> PromptManager for PromptSvc<S> {
    async fn create_new(
        &self,
        author_id: i32,
        body: &str,
    ) -> Result<PromptWithAuthor, DomainError> {
        match self.store.insert_new(author_id, body).await {
            Err(InsertionError::Technical(e)) => Err(TechnicalError::Database(e).into()),
            Err(InsertionError::UniqueViolation(_)) => Err(ContentError::DuplicatePrompt.into()),
            Ok(id) => Ok(PromptWithAuthor {
                id,
                author_username: self.user_svc.get_by_id(author_id).await?.username,
                body: body.to_string(),
            }),
        }
    }

    async fn get_for_writing(
        &self,
        requester_id: i32,
        prompt_id: i32,
    ) -> Result<PromptWithAuthor, DomainError> {
        let prompt = self
            .store
            .get_by_id(prompt_id)
            .await?
            .ok_or(ContentError::NotFound)?;

        // Disallow writing a post in response to one's own prompt
        if requester_id == prompt.author_id {
            return Err(ContentError::OwnPrompt.into());
        }

        // Must be friends to see someone's prompts
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
            Err(ContentError::NotFriends.into())
        }
    }

    async fn single_user_prompts(
        &self,
        user_id: i32,
    ) -> Result<Vec<PromptWithAuthor>, DomainError> {
        self.store
            .single_user_prompts(user_id)
            .await
            .map_err(DomainError::from)
    }

    async fn all_friend_prompts(&self, user_id: i32) -> Result<Vec<PromptWithAuthor>, DomainError> {
        self.store
            .all_friend_prompts(user_id)
            .await
            .map_err(DomainError::from)
    }
}
