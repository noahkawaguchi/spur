use crate::{
    app_services::{
        MutateFriendshipByUsername,
        mutate_friendship_by_username_svc::MutateFriendshipByUsernameSvc,
    },
    domain::{
        post::{PostSvc, service::PostDomainSvc},
        user::{UserSvc, service::UserDomainSvc},
    },
    infra::{
        friendship_repo::PgFriendshipRepo, post_repo::PgPostRepo,
        post_with_author_read::PgPostWithAuthorRead, social_read::PgSocialRead,
        user_repo::PgUserRepo,
    },
    read_models::{PostWithAuthorRead, SocialRead},
};
use axum::extract::FromRef;
use std::sync::Arc;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub jwt_secret: String,
    pub user_svc: Arc<dyn UserSvc>,
    pub mutate_friendship_by_username: Arc<dyn MutateFriendshipByUsername>,
    pub post_svc: Arc<dyn PostSvc>,
    pub social_read: Arc<dyn SocialRead>,
    pub post_with_author_read: Arc<dyn PostWithAuthorRead>,
}

impl AppState {
    /// Wires together the repository and service layers for use as `State` in routers/handlers.
    pub fn build(pool: sqlx::PgPool, jwt_secret: String) -> Self {
        let user_svc = Arc::new(UserDomainSvc::new(pool.clone(), PgUserRepo)) as Arc<dyn UserSvc>;

        let mutate_friendship_by_username = Arc::new(MutateFriendshipByUsernameSvc::new(
            pool.clone(),
            PgUserRepo,
            PgFriendshipRepo,
        )) as Arc<dyn MutateFriendshipByUsername>;

        let post_svc =
            Arc::new(PostDomainSvc::new(PgPostRepo::new(pool.clone()))) as Arc<dyn PostSvc>;

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

#[cfg(test)]
impl Default for AppState {
    fn default() -> Self {
        use crate::{
            app_services::MockMutateFriendshipByUsername,
            domain::{post::MockPostSvc, user::MockUserSvc},
            read_models::{MockPostWithAuthorRead, MockSocialRead},
        };

        Self {
            jwt_secret: String::from("top_secret"),
            user_svc: Arc::new(MockUserSvc::new()),
            mutate_friendship_by_username: Arc::new(MockMutateFriendshipByUsername::new()),
            post_svc: Arc::new(MockPostSvc::new()),
            social_read: Arc::new(MockSocialRead::new()),
            post_with_author_read: Arc::new(MockPostWithAuthorRead::new()),
        }
    }
}
