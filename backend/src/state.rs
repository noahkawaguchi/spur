use crate::{
    domain::{friendship::service::FriendshipManager, post::PostManager, user::UserManager},
    infra::{post_with_author_read::PgPostWithAuthorRead, social_read::PgSocialRead},
    read_models::{PostWithAuthorRead, SocialRead},
    repository::{friendship::PgFriendshipRepo, post::PgPostRepo, user::PgUserRepo},
    service::{friendship::FriendshipSvc, post::PostSvc, user::UserSvc},
};
use axum::extract::FromRef;
use std::sync::Arc;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub jwt_secret: String,
    pub user_svc: Arc<dyn UserManager>,
    pub friendship_svc: Arc<dyn FriendshipManager>,
    pub post_svc: Arc<dyn PostManager>,
    pub social_read: Arc<dyn SocialRead>,
    pub post_with_author_read: Arc<dyn PostWithAuthorRead>,
}

impl AppState {
    /// Wires together the repository and service layers for use as `State` in routers/handlers.
    pub fn build(pool: sqlx::PgPool, jwt_secret: String) -> Self {
        let user_svc =
            Arc::new(UserSvc::new(PgUserRepo::new(pool.clone()))) as Arc<dyn UserManager>;

        let friendship_svc = Arc::new(FriendshipSvc::new(
            pool.clone(),
            PgFriendshipRepo,
            Arc::clone(&user_svc),
        )) as Arc<dyn FriendshipManager>;

        let post_svc =
            Arc::new(PostSvc::new(PgPostRepo::new(pool.clone()))) as Arc<dyn PostManager>;

        let social_read = Arc::new(PgSocialRead::new(pool.clone())) as Arc<dyn SocialRead>;
        let post_with_author_read =
            Arc::new(PgPostWithAuthorRead::new(pool)) as Arc<dyn PostWithAuthorRead>;

        Self { jwt_secret, user_svc, friendship_svc, post_svc, social_read, post_with_author_read }
    }
}
