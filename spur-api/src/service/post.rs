use crate::{
    domain::{
        content::{
            error::ContentError,
            repository::PostStore,
            service::{PostManager, PromptManager},
        },
        error::DomainError,
        friendship::{service::FriendshipManager, user_id_pair::UserIdPair},
    },
    repository::insertion_error::InsertionError,
    technical_error::TechnicalError,
    utils::vec_into,
};
use spur_shared::models::PostWithPrompt;
use std::sync::Arc;

pub struct PostSvc<S: PostStore> {
    store: S,
    friendship_svc: Arc<dyn FriendshipManager>,
    prompt_svc: Arc<dyn PromptManager>,
}

impl<S: PostStore> PostSvc<S> {
    pub const fn new(
        store: S,
        friendship_svc: Arc<dyn FriendshipManager>,
        prompt_svc: Arc<dyn PromptManager>,
    ) -> Self {
        Self { store, friendship_svc, prompt_svc }
    }
}

#[async_trait::async_trait]
impl<S: PostStore> PostManager for PostSvc<S> {
    async fn create_new(
        &self,
        author_id: i32,
        prompt_id: i32,
        body: &str,
    ) -> Result<PostWithPrompt, DomainError> {
        // Prompt must exist and be written by a friend to be able to write a post in response
        self.prompt_svc
            .get_for_writing(author_id, prompt_id)
            .await?;

        match self.store.insert_new(author_id, prompt_id, body).await {
            Err(InsertionError::Technical(e)) => Err(TechnicalError::Database(e).into()),
            Err(InsertionError::UniqueViolation(_)) => Err(ContentError::DuplicatePost.into()),
            Ok(post) => Ok(post.into()),
        }
    }

    async fn get_for_reading(
        &self,
        requester_id: i32,
        post_id: i32,
    ) -> Result<PostWithPrompt, DomainError> {
        // Post must exist
        let post = self
            .store
            .get_by_id(post_id)
            .await?
            .ok_or(ContentError::NotFound)?;

        // Only allow reading posts written by oneself or one's friends
        if requester_id == post.author_id
            || self
                .friendship_svc
                .are_friends(&UserIdPair::new(requester_id, post.author_id)?)
                .await?
        {
            Ok(post.into())
        } else {
            Err(ContentError::NotFriends.into())
        }
    }

    async fn single_user_posts(&self, author_id: i32) -> Result<Vec<PostWithPrompt>, DomainError> {
        self.store
            .single_user_posts(author_id)
            .await
            .map_err(Into::into)
            .map(vec_into)
    }

    async fn all_friend_posts(&self, user_id: i32) -> Result<Vec<PostWithPrompt>, DomainError> {
        self.store
            .all_friend_posts(user_id)
            .await
            .map_err(Into::into)
            .map(vec_into)
    }
}
