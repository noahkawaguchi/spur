use crate::{
    domain::{
        error::DomainError,
        friendship::{service::FriendshipManager, user_id_pair::UserIdPair},
        prompt::{ContentError, ContentManager, PromptStore},
        user::UserManager,
    },
    repository::insertion_error::InsertionError,
    technical_error::TechnicalError,
};
use spur_shared::models::PromptWithAuthor;
use std::sync::Arc;

pub struct ContentSvc<S: PromptStore> {
    store: S,
    friendship_svc: Arc<dyn FriendshipManager>,
    user_svc: Arc<dyn UserManager>,
}

impl<S: PromptStore> ContentSvc<S> {
    pub const fn new(
        store: S,
        friendship_svc: Arc<dyn FriendshipManager>,
        user_svc: Arc<dyn UserManager>,
    ) -> Self {
        Self { store, friendship_svc, user_svc }
    }
}

#[async_trait::async_trait]
impl<S: PromptStore> ContentManager for ContentSvc<S> {
    async fn new_prompt(
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

    async fn get_prompt_for_writing(
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

    async fn own_prompts(&self, user_id: i32) -> Result<Vec<PromptWithAuthor>, DomainError> {
        self.store
            .get_user_prompts(user_id)
            .await
            .map_err(DomainError::from)
    }

    async fn specific_friend_prompts(
        &self,
        requester_id: i32,
        friend_username: &str,
    ) -> Result<Vec<PromptWithAuthor>, DomainError> {
        let friend_id = self.user_svc.get_by_username(friend_username).await?.id;

        // Must be friends to see someone's prompts
        if self
            .friendship_svc
            .are_friends(&UserIdPair::new(requester_id, friend_id)?)
            .await?
        {
            self.store
                .get_user_prompts(friend_id)
                .await
                .map_err(DomainError::from)
        } else {
            Err(ContentError::NotFriends.into())
        }
    }

    async fn all_friend_prompts(&self, user_id: i32) -> Result<Vec<PromptWithAuthor>, DomainError> {
        self.store
            .get_friend_prompts(user_id)
            .await
            .map_err(DomainError::from)
    }
}
