use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid password")]
    InvalidPassword,
    #[error("Expired or invalid token. Try logging in again.")]
    JwtValidation,
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    exp: u64,
}

impl Claims {
    /// Initializes claims with an expiration 24 hours in the future.
    pub fn new(id: i32) -> Result<Self> {
        let now = Utc::now();

        let exp = (now + Duration::hours(24))
            .timestamp()
            .try_into()
            .with_context(|| format!("pre-1970 system time: {now}"))?;

        Ok(Self { sub: id.to_string(), exp })
    }

    /// Attempts to parse the subject as an i32.
    pub fn parse_sub(&self) -> Result<i32, std::num::ParseIntError> { self.sub.parse() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::time::within_one_second;
    use chrono::{DateTime, Days};

    #[test]
    fn converts_types_and_calculates_expiration() {
        let id = 825;
        let tomorrow = Utc::now()
            .checked_add_days(Days::new(1))
            .expect("failed to compute tomorrow");

        let claims = Claims::new(id).expect("failed to create claims");

        let exp = DateTime::from_timestamp(
            claims.exp.try_into().expect("failed to convert u64 to i64"),
            0,
        )
        .expect("failed to create datetime");

        assert!(within_one_second(exp, tomorrow));
        assert_eq!(claims.sub, id.to_string());
    }
}
