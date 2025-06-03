use crate::domain::{
    content::{
        error::ContentError,
        service::{ContentManager, PostManager, PromptManager},
    },
    error::DomainError,
    friendship::{service::FriendshipManager, user_id_pair::UserIdPair},
    user::UserManager,
};
use spur_shared::models::{PostWithPrompt, PromptWithAuthor};
use std::sync::Arc;

pub struct ContentSvc {
    users: Arc<dyn UserManager>,
    friendships: Arc<dyn FriendshipManager>,
    prompts: Arc<dyn PromptManager>,
    posts: Arc<dyn PostManager>,
}

impl ContentSvc {
    pub const fn new(
        users: Arc<dyn UserManager>,
        friendships: Arc<dyn FriendshipManager>,
        prompts: Arc<dyn PromptManager>,
        posts: Arc<dyn PostManager>,
    ) -> Self {
        Self { users, friendships, prompts, posts }
    }
}

#[async_trait::async_trait]
impl ContentManager for ContentSvc {
    async fn own_content(
        &self,
        user_id: i32,
    ) -> Result<(Vec<PromptWithAuthor>, Vec<PostWithPrompt>), DomainError> {
        let prompts = self.prompts.single_user_prompts(user_id).await?;
        let posts = self.posts.single_user_posts(user_id).await?;
        Ok((prompts, posts))
    }

    async fn specific_friend_content(
        &self,
        requester_id: i32,
        friend_username: &str,
    ) -> Result<(Vec<PromptWithAuthor>, Vec<PostWithPrompt>), DomainError> {
        let friend_id = self.users.get_by_username(friend_username).await?.id;

        // Must be friends to see someone's content
        if self
            .friendships
            .are_friends(&UserIdPair::new(requester_id, friend_id)?)
            .await?
        {
            let prompts = self.prompts.single_user_prompts(friend_id).await?;
            let posts = self.posts.single_user_posts(friend_id).await?;
            Ok((prompts, posts))
        } else {
            Err(ContentError::NotFriends.into())
        }
    }

    async fn all_friend_content(
        &self,
        user_id: i32,
    ) -> Result<(Vec<PromptWithAuthor>, Vec<PostWithPrompt>), DomainError> {
        let prompts = self.prompts.all_friend_prompts(user_id).await?;
        let posts = self.posts.all_friend_posts(user_id).await?;
        Ok((prompts, posts))
    }
}
