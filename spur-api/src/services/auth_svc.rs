use crate::{
    handlers::auth_handlers::AuthService,
    models::{NewUser, User},
};
use anyhow::Result;
use spur_shared::dto::{LoginRequest, SignupRequest};

#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    async fn insert_new(&self, new_user: &NewUser<'_>) -> sqlx::Result<()>;
    async fn get_by_email(&self, email: &str) -> sqlx::Result<User>;
    async fn get_by_username(&self, username: &str) -> sqlx::Result<User>;
}

#[derive(Clone)]
pub struct AuthSvc<R: UserRepository> {
    repo: R,
}

impl<R: UserRepository> AuthSvc<R> {
    pub const fn new(repo: R) -> Self { Self { repo } }
}

#[async_trait::async_trait]
impl<R: UserRepository> AuthService for AuthSvc<R> {
    /// Checks if an account with the given email or username already exists in the database.
    async fn email_username_available(&self, req: &SignupRequest) -> Result<(), String> {
        if self.repo.get_by_email(&req.email).await.is_ok() {
            return Err(String::from(
                "an account with the same email already exists",
            ));
        }

        if self.repo.get_by_username(&req.username).await.is_ok() {
            return Err(String::from(
                "an account with the same username already exists",
            ));
        }

        Ok(())
    }

    /// Hashes the password and creates a new user in the database.
    async fn register(&self, req: &SignupRequest) -> Result<()> {
        let hashed = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)?;
        let new_user = NewUser::from_request(req, &hashed);
        self.repo.insert_new(&new_user).await?;
        Ok(())
    }

    /// Checks `email` and `password` for a valid match in the database.
    async fn validate_credentials(&self, req: &LoginRequest) -> Result<User, String> {
        // Check if the user exists
        let Ok(user) = self.repo.get_by_email(&req.email).await else {
            return Err(String::from("invalid email"));
        };

        // Validate the password
        if !bcrypt::verify(&req.password, &user.password_hash).is_ok_and(|is_valid| is_valid) {
            return Err(String::from("invalid password"));
        }

        Ok(user)
    }
}
