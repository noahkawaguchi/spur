use crate::{
    app_services::{
        Authenticator, MutateFriendshipByUsername, authenticator_svc::AuthenticatorSvc,
        mutate_friendship_by_username_svc::MutateFriendshipByUsernameSvc,
    },
    domain::post::{PostSvc, service::PostDomainSvc},
    infra::{
        auth_provider::BcryptJwtAuthProvider, friendship_repo::PgFriendshipRepo,
        post_repo::PgPostRepo, post_with_author_read::PgPostWithAuthorRead,
        social_read::PgSocialRead, user_repo::PgUserRepo,
    },
    read_models::{PostWithAuthorRead, SocialRead},
};
use axum::extract::FromRef;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub auth: Arc<dyn Authenticator>,
    pub mutate_friendship_by_username: Arc<dyn MutateFriendshipByUsername>,
    pub post_svc: Arc<dyn PostSvc>,
    pub social_read: Arc<dyn SocialRead>,
    pub post_with_author_read: Arc<dyn PostWithAuthorRead>,
}

impl AppState {
    /// Wires together the repository and service layers for use as `State` in handlers/middleware.
    pub fn build(pool: PgPool, jwt_secret: String) -> Self {
        let auth = Arc::new(AuthenticatorSvc::new(
            pool.clone(),
            PgUserRepo,
            BcryptJwtAuthProvider::new(jwt_secret),
        ));

        let mutate_friendship_by_username = Arc::new(MutateFriendshipByUsernameSvc::new(
            pool.clone(),
            PgUserRepo,
            PgFriendshipRepo,
        )) as Arc<dyn MutateFriendshipByUsername>;

        let post_svc = Arc::new(PostDomainSvc::new(pool.clone(), PgPostRepo)) as Arc<dyn PostSvc>;

        let social_read = Arc::new(PgSocialRead::new(pool.clone())) as Arc<dyn SocialRead>;
        let post_with_author_read =
            Arc::new(PgPostWithAuthorRead::new(pool)) as Arc<dyn PostWithAuthorRead>;

        Self { auth, mutate_friendship_by_username, post_svc, social_read, post_with_author_read }
    }
}

#[cfg(test)]
impl Default for AppState {
    fn default() -> Self {
        use crate::{
            app_services::{MockAuthenticator, MockMutateFriendshipByUsername},
            domain::post::MockPostSvc,
            read_models::{MockPostWithAuthorRead, MockSocialRead},
        };

        Self {
            auth: Arc::new(MockAuthenticator::new()),
            mutate_friendship_by_username: Arc::new(MockMutateFriendshipByUsername::new()),
            post_svc: Arc::new(MockPostSvc::new()),
            social_read: Arc::new(MockSocialRead::new()),
            post_with_author_read: Arc::new(MockPostWithAuthorRead::new()),
        }
    }
}
