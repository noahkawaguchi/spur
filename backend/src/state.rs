use crate::{
    app_services::{
        MutateFriendshipByUsername,
        mutate_friendship_by_username_svc::MutateFriendshipByUsernameSvc,
    },
    domain::{post::PostManager, user::UserManager},
    infra::{post_with_author_read::PgPostWithAuthorRead, social_read::PgSocialRead},
    read_models::{PostWithAuthorRead, SocialRead},
    repository::{friendship::PgFriendshipRepo, post::PgPostRepo, user::PgUserRepo},
    service::{post::PostSvc, user::UserSvc},
};
use axum::extract::FromRef;
use std::sync::Arc;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub jwt_secret: String,
    pub user_svc: Arc<dyn UserManager>,
    pub mutate_friendship_by_username: Arc<dyn MutateFriendshipByUsername>,
    pub post_svc: Arc<dyn PostManager>,
    pub social_read: Arc<dyn SocialRead>,
    pub post_with_author_read: Arc<dyn PostWithAuthorRead>,
}

impl AppState {
    /// Wires together the repository and service layers for use as `State` in routers/handlers.
    pub fn build(pool: sqlx::PgPool, jwt_secret: String) -> Self {
        let user_svc = Arc::new(UserSvc::new(pool.clone(), PgUserRepo)) as Arc<dyn UserManager>;

        let mutate_friendship_by_username = Arc::new(MutateFriendshipByUsernameSvc::new(
            pool.clone(),
            PgUserRepo,
            PgFriendshipRepo,
        )) as Arc<dyn MutateFriendshipByUsername>;

        let post_svc =
            Arc::new(PostSvc::new(PgPostRepo::new(pool.clone()))) as Arc<dyn PostManager>;

        let social_read = Arc::new(PgSocialRead::new(pool.clone())) as Arc<dyn SocialRead>;
        let post_with_author_read =
            Arc::new(PgPostWithAuthorRead::new(pool)) as Arc<dyn PostWithAuthorRead>;

        Self {
            jwt_secret,
            user_svc,
            mutate_friendship_by_username,
            post_svc,
            social_read,
            post_with_author_read,
        }
    }
}
