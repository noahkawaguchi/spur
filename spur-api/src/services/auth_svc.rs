use crate::{
    models::{NewUser, User},
    repositories::user_repo,
};
use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use spur_shared::dto::{LoginRequest, SignupRequest};
use sqlx::PgPool;

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: u64,
}

impl Claims {
    /// Initializes claims with an expiration 24 hours in the future.
    fn new(id: i32) -> Result<Self> {
        let now = Utc::now();
        let exp = (now + Duration::hours(24))
            .timestamp()
            .try_into()
            .context(format!("unexpected pre-1970 system time: {now}"))?;
        Ok(Self { sub: id.to_string(), exp })
    }
}

/// Checks if an account with the given email or username already exists in the database.
pub async fn email_username_available(pool: &PgPool, req: &SignupRequest) -> Result<(), String> {
    if user_repo::get_by_email(pool, &req.email).await.is_ok() {
        return Err(String::from(
            "an account with the same email already exists",
        ));
    }

    if user_repo::get_by_username(pool, &req.username)
        .await
        .is_ok()
    {
        return Err(String::from(
            "an account with the same username already exists",
        ));
    }

    Ok(())
}

/// Hashes the password and creates a new user in the database.
pub async fn register(pool: &PgPool, req: &SignupRequest) -> Result<()> {
    let hashed = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)?;
    let new_user = NewUser::from_request(req, &hashed);
    user_repo::insert_new(pool, &new_user).await?;
    Ok(())
}

/// Checks `email` and `password` for a valid match in the database.
pub async fn validate_credentials(pool: &PgPool, req: &LoginRequest) -> Result<User, String> {
    // Check if the user exists
    let Ok(user) = user_repo::get_by_email(pool, &req.email).await else {
        return Err(String::from("invalid email"));
    };

    // Validate the password
    if !bcrypt::verify(&req.password, &user.password_hash).is_ok_and(|is_valid| is_valid) {
        return Err(String::from("invalid password"));
    }

    Ok(user)
}

/// Creates a JSON web token with the id as the subject.
pub fn create_jwt(id: i32, secret: &[u8]) -> Result<String> {
    let token = jsonwebtoken::encode(
        &Header::default(),
        &Claims::new(id)?,
        &EncodingKey::from_secret(secret),
    )?;

    Ok(token)
}

/// Validates a JSON web token and parses the user ID.
pub fn verify_jwt(token: &str, secret: &[u8]) -> Result<i32> {
    let id = jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret),
        &Validation::default(),
    )?
    .claims
    .sub
    .parse()?;

    Ok(id)
}
