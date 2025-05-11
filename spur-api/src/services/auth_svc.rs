use crate::{
    models::NewUser,
    repositories::user_repo::{self, get_by_email, get_by_username},
};
use anyhow::Result;
use bcrypt::{DEFAULT_COST, hash};
use spur_shared::dto::SignupRequest;
use sqlx::PgPool;

/// Checks if an account with the given email or username already exists in the database.
pub async fn email_username_taken(pool: &PgPool, req: &SignupRequest) -> Result<(), String> {
    if get_by_email(pool, &req.email).await.is_ok() {
        return Err(String::from(
            "an account with the same email already exists",
        ));
    }

    if get_by_username(pool, &req.username).await.is_ok() {
        return Err(String::from(
            "an account with the same username already exists",
        ));
    }

    Ok(())
}

/// Hashes the password and creates a new user in the database.
pub async fn signup(pool: &PgPool, req: &SignupRequest) -> Result<()> {
    let hashed = hash(&req.password, DEFAULT_COST)?;
    let new_user = NewUser::from_request(req, &hashed);
    user_repo::insert_new(pool, &new_user).await?;
    Ok(())
}
