use crate::{
    domain::{friendship::service::FriendshipManager, post::PostManager, user::UserManager},
    repository::{friendship::FriendshipRepo, post::PostRepo, user::UserRepo},
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
}

impl AppState {
    /// Wires together the repository and service layers for use as `State` in routers/handlers.
    pub fn build(pool: sqlx::PgPool, jwt_secret: String) -> Self {
        let user_svc = Arc::new(UserSvc::new(UserRepo::new(pool.clone()))) as Arc<dyn UserManager>;

        let friendship_svc = Arc::new(FriendshipSvc::new(
            pool.clone(),
            FriendshipRepo,
            Arc::clone(&user_svc),
        )) as Arc<dyn FriendshipManager>;

        let post_svc = Arc::new(PostSvc::new(PostRepo::new(pool))) as Arc<dyn PostManager>;

        Self { jwt_secret, user_svc, friendship_svc, post_svc }
    }
}
