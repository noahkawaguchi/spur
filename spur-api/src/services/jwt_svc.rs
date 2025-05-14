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

#[cfg(test)]
mod tests {
    use super::*;

    mod claims {
        use super::*;
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

            assert!((exp - tomorrow).num_seconds().abs() <= 1);
            assert_eq!(claims.sub, id.to_string());
        }
    }

    mod tokens {
        use super::*;

        #[test]
        fn encodes_and_decodes() {
            let id = 783;
            let secret = "super_duper_secret".as_ref();

            let token = create_jwt(id, secret).expect("failed to create token");
            let got_id = verify_jwt(&token, secret).expect("failed to verify token");

            assert_eq!(got_id, id);
        }

        #[test]
        fn errors_for_invalid_token() {
            let id = 4621;
            let secret = "no one's gonna know".as_ref();

            let _ = create_jwt(id, secret).expect("failed to create token");
            assert!(verify_jwt("fake token", secret).is_err());
        }

        #[test]
        fn errors_for_invalid_secret() {
            let id = 999;
            let secret = "shh".as_ref();

            let token = create_jwt(id, secret).expect("failed to create token");
            assert!(verify_jwt(&token, "boo!".as_ref()).is_err());
        }
    }
}
