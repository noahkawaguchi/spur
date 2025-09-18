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

    // fn make_bob(password: &str) -> User {
    //     User {
    //         id: 55,
    //         name: String::from("Bobby Robertson"),
    //         email: String::from("rob@bob.net"),
    //         username: String::from("amazing_robby_7"),
    //         password_hash: bcrypt::hash(password, bcrypt::DEFAULT_COST)
    //             .expect("failed to hash password"),
    //         created_at: Utc::now()
    //             .checked_sub_days(Days::new(1))
    //             .expect("failed to subtract a day from now"),
    //     }
    // }
    //
    // mod provider {
    //     use super::*;
    //
    //     #[test]
    //     fn encodes_and_decodes_id_for_valid_pw() {
    //         let correct_pw = "Bob's password";
    //         let secret = "shh_hhh_hhh";
    //
    //         let bob = make_bob(correct_pw);
    //         let token = create_jwt_if_valid_pw(&bob, correct_pw, secret)
    //             .expect("failed to create JWT for valid password");
    //
    //         let id = validate_jwt(&token, secret).expect("failed to validate token");
    //         assert_eq!(id, bob.id);
    //     }
    //
    //     #[test]
    //     fn token_creation_errors_for_invalid_pw() {
    //         let bob = make_bob("correct password");
    //         let result = create_jwt_if_valid_pw(&bob, "incorrect password", "top secret");
    //         assert!(matches!(result, Err(AuthError::InvalidPassword)));
    //     }
    //
    //     #[test]
    //     fn validation_errors_for_invalid_token() {
    //         let password = "53cur1ty";
    //         let bob = make_bob(password);
    //         let secret = "no one's gonna know";
    //
    //         let _ = create_jwt_if_valid_pw(&bob, password, secret).expect("failed to create
    // token");         assert!(matches!(
    //             validate_jwt("fake token", secret),
    //             Err(AuthError::TokenValidation),
    //         ));
    //     }
    //
    //     #[test]
    //     fn validation_errors_for_invalid_secret() {
    //         let password = "pa$$ed654wood&24b1";
    //         let bob = make_bob(password);
    //         let secret = "shh";
    //
    //         let token =
    //             create_jwt_if_valid_pw(&bob, password, secret).expect("failed to create token");
    //         assert!(matches!(
    //             validate_jwt(&token, "boo!"),
    //             Err(AuthError::TokenValidation)
    //         ));
    //     }
    // }

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
