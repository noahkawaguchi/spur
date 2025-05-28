use crate::domain::{friendship::service::FriendshipManager, user::UserManager};
use anyhow::{Context, Result};
use axum::extract::FromRef;
use std::{env, sync::Arc};

pub struct AppConfig {
    pub database_url: String,
    pub bind_addr: String,
    pub jwt_secret: String,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        // Expect a .env file in development only
        if cfg!(debug_assertions) {
            dotenvy::dotenv()?;
        }

        let database_url = env::var("DATABASE_URL").context("failed to load DATABASE_URL")?;
        let bind_addr = env::var("BIND_ADDR").context("failed to load BIND_ADDR")?;
        let jwt_secret = env::var("JWT_SECRET").context("failed to load JWT_SECRET")?;

        Ok(Self { database_url, bind_addr, jwt_secret })
    }
}

#[derive(Clone, FromRef)]
pub struct AppState {
    pub jwt_secret: String,
    pub user_svc: Arc<dyn UserManager>,
    pub friendship_svc: Arc<dyn FriendshipManager>,
}
