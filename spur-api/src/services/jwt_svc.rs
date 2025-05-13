use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

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
