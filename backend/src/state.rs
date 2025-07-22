use crate::{
    domain::{
        content::service::{ContentManager, PostManager, PromptManager},
        friendship::service::FriendshipManager,
        user::UserManager,
    },
    repository::{friendship::FriendshipRepo, post::PostRepo, prompt::PromptRepo, user::UserRepo},
    service::{
        content::ContentSvc, friendship::FriendshipSvc, post::PostSvc, prompt::PromptSvc,
        user::UserSvc,
    },
};
use axum::extract::FromRef;
use std::sync::Arc;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub jwt_secret: String,
    pub user_svc: Arc<dyn UserManager>,
    pub friendship_svc: Arc<dyn FriendshipManager>,
    pub prompt_svc: Arc<dyn PromptManager>,
    pub post_svc: Arc<dyn PostManager>,
    pub content_svc: Arc<dyn ContentManager>,
}

impl AppState {
    /// Wires together the repository and service layers for use as `State` in routers/handlers.
    pub fn build(pool: sqlx::PgPool, jwt_secret: String) -> Self {
        let user_svc = Arc::new(UserSvc::new(UserRepo::new(pool.clone()))) as Arc<dyn UserManager>;

        let friendship_svc = Arc::new(FriendshipSvc::new(
            FriendshipRepo::new(pool.clone()),
            Arc::clone(&user_svc),
        )) as Arc<dyn FriendshipManager>;

        let prompt_svc = Arc::new(PromptSvc::new(
            PromptRepo::new(pool.clone()),
            Arc::clone(&friendship_svc),
        )) as Arc<dyn PromptManager>;

        let post_svc = Arc::new(PostSvc::new(
            PostRepo::new(pool),
            Arc::clone(&friendship_svc),
            Arc::clone(&prompt_svc),
        )) as Arc<dyn PostManager>;

        let content_svc = Arc::new(ContentSvc::new(
            Arc::clone(&user_svc),
            Arc::clone(&friendship_svc),
            Arc::clone(&prompt_svc),
            Arc::clone(&post_svc),
        ));

        Self { jwt_secret, user_svc, friendship_svc, prompt_svc, post_svc, content_svc }
    }
}
