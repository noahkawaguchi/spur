use crate::{
    domain::{auth::AuthError, error::DomainError},
    models::user::{NewUser, User, UserRegistration},
    technical_error::TechnicalError,
};
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
    fn new(id: i32) -> Result<Self, TechnicalError> {
        let now = Utc::now();

        let exp = (now + Duration::hours(24))
            .timestamp()
            .try_into()
            .map_err(|_| TechnicalError::Pre1970(now))?;

        Ok(Self { sub: id.to_string(), exp })
    }
}

pub fn hash_pw(reg: UserRegistration) -> Result<NewUser, DomainError> {
    let pw_hash =
        bcrypt::hash(&reg.password, bcrypt::DEFAULT_COST).map_err(TechnicalError::from)?;

    Ok(reg.into_new_user_with(pw_hash))
}

pub fn jwt_if_valid_pw(user: &User, password: &str, secret: &str) -> Result<String, DomainError> {
    // Validate the password
    bcrypt::verify(password, &user.password_hash)
        .map_err(TechnicalError::from)?
        .then_some(())
        .ok_or(AuthError::InvalidPassword)?;

    // Create the JWT
    let token = jsonwebtoken::encode(
        &Header::default(),
        &Claims::new(user.id)?,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(TechnicalError::from)?;

    Ok(token)
}

pub fn validate_jwt(token: &str, secret: &str) -> Result<i32, DomainError> {
    if let Ok(token_data) = jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    ) {
        if let Ok(id) = token_data.claims.sub.parse() {
            return Ok(id);
        }
    }

    Err(AuthError::JwtValidation.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::within_one_second;
    use chrono::{DateTime, Days};
    use colored::Colorize;

    fn make_bob(password: &str) -> User {
        User {
            id: 55,
            name: String::from("Bobby Robertson"),
            email: String::from("rob@bob.net"),
            username: String::from("amazing_robby_7"),
            password_hash: bcrypt::hash(password, bcrypt::DEFAULT_COST)
                .expect("failed to hash password"),
            created_at: Utc::now()
                .checked_sub_days(Days::new(1))
                .expect("failed to subtract a day from now"),
        }
    }

    #[test]
    fn claims_converts_types_and_calculates_expiration() {
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

    // Determined that testing hash_pw would be trivial

    #[test]
    fn encodes_and_decodes_id_for_valid_pw() {
        let correct_pw = "Bob's password";
        let secret = "shh_hhh_hhh";

        let bob = make_bob(correct_pw);
        let token = jwt_if_valid_pw(&bob, correct_pw, secret)
            .expect("failed to create JWT for valid password");

        let id = validate_jwt(&token, secret).expect("failed to validate token");
        assert_eq!(id, bob.id);
    }

    #[test]
    fn token_creation_errors_for_invalid_pw() {
        let bob = make_bob("correct password");
        let result = jwt_if_valid_pw(&bob, "incorrect password", "top secret");
        assert!(
            matches!(result, Err(DomainError::Auth(AuthError::InvalidPassword))),
            "{}",
            format!("{result:?}").red(),
        );
    }

    #[test]
    fn validation_errors_for_invalid_token() {
        let password = "53cur1ty";
        let bob = make_bob(password);
        let secret = "no one's gonna know";

        let _ = jwt_if_valid_pw(&bob, password, secret).expect("failed to create token");
        assert!(matches!(
            validate_jwt("fake token", secret),
            Err(DomainError::Auth(AuthError::JwtValidation)),
        ));
    }

    #[test]
    fn validation_errors_for_invalid_secret() {
        let password = "pa$$ed654wood&24b1";
        let bob = make_bob(password);
        let secret = "shh";

        let token = jwt_if_valid_pw(&bob, password, secret).expect("failed to create token");
        assert!(matches!(
            validate_jwt(&token, "boo!"),
            Err(DomainError::Auth(AuthError::JwtValidation)),
        ));
    }
}
