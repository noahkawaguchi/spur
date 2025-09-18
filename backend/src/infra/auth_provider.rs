use crate::domain::auth::AuthProvider;
use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

pub struct BcryptJwtAuthProvider {
    jwt_secret: String,
}

impl BcryptJwtAuthProvider {
    pub const fn new(jwt_secret: String) -> Self { Self { jwt_secret } }
}

impl AuthProvider for BcryptJwtAuthProvider {
    fn hash_pw(&self, pw: &str) -> Result<String> {
        bcrypt::hash(pw, bcrypt::DEFAULT_COST).context("failed to hash password")
    }

    fn is_valid_pw(&self, pw: &str, hash: &str) -> Result<bool> {
        bcrypt::verify(pw, hash).context("failed to verify password hash")
    }

    fn create_token(&self, payload: i32) -> Result<String> {
        jsonwebtoken::encode(
            &Header::default(),
            &Claims::new(payload)?,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )
        .context("failed to create JWT")
    }

    fn validate_token(&self, token: &str) -> Result<i32> {
        jsonwebtoken::decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &Validation::default(),
        )?
        .claims
        .parse_sub()
    }
}

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: u64,
}

impl Claims {
    /// Initializes claims with an expiration 24 hours in the future.
    fn new(payload: i32) -> Result<Self> {
        let now = Utc::now();

        let exp = (now + Duration::hours(24))
            .timestamp()
            .try_into()
            .with_context(|| format!("pre-1970 system time: {now}"))?;

        Ok(Self { sub: payload.to_string(), exp })
    }
    /// Attempts to parse the subject as an i32.
    fn parse_sub(&self) -> Result<i32> { self.sub.parse().map_err(Into::into) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::time::within_one_second;
    use chrono::{DateTime, Days, Utc};

    const TEST_JWT_SECRET: &str = "super_secret_testing_only";

    mod password_hashing {
        use super::*;

        #[test]
        fn hashes_and_validates_valid_password() {
            let password = "password123";
            let auth = BcryptJwtAuthProvider::new(TEST_JWT_SECRET.to_string());
            let hash = auth.hash_pw(password).unwrap();
            assert!(auth.is_valid_pw(password, &hash).unwrap());
        }

        #[test]
        fn identifies_invalid_password() {
            let auth = BcryptJwtAuthProvider::new(TEST_JWT_SECRET.to_string());
            let hash = auth.hash_pw("correct password").unwrap();
            assert!(!auth.is_valid_pw("incorrect!", &hash).unwrap());
        }

        #[test]
        fn identifies_invalid_hash() {
            let password = "this password is correct";
            let auth = BcryptJwtAuthProvider::new(TEST_JWT_SECRET.to_string());
            let _correct_hash = auth.hash_pw(password).unwrap();
            let incorrect_hash = auth.hash_pw("some other password").unwrap();
            assert!(!auth.is_valid_pw(password, &incorrect_hash).unwrap());
        }
    }

    mod token_validation {
        use super::*;

        #[test]
        fn creates_and_validates_valid_token() {
            let user_id = 25_925;
            let auth = BcryptJwtAuthProvider::new(TEST_JWT_SECRET.to_string());
            let token = auth.create_token(user_id).unwrap();
            assert_eq!(user_id, auth.validate_token(&token).unwrap());
        }

        #[test]
        fn identifies_invalid_token() {
            let auth = BcryptJwtAuthProvider::new(TEST_JWT_SECRET.to_string());
            let _correct_token = auth.create_token(5432).unwrap();
            assert!(auth.validate_token("not correct").is_err());
        }

        #[test]
        fn creates_different_tokens_for_different_ids() {
            let (user_id_1, user_id_2) = (42, 43);
            let auth = BcryptJwtAuthProvider::new(TEST_JWT_SECRET.to_string());
            let _token_1 = auth.create_token(user_id_1).unwrap();
            let token_2 = auth.create_token(user_id_2).unwrap();
            assert_ne!(user_id_1, auth.validate_token(&token_2).unwrap());
        }
    }

    mod claims {
        use super::*;

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
}
