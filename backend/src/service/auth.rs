use crate::{
    domain::auth::{AuthError, Claims},
    models::user::User,
};
use anyhow::Context;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};

pub fn hash_pw(password: &str) -> Result<String, AuthError> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .context("failed to hash password")
        .map_err(Into::into)
}

/// Creates a JSON Web Token after verifying the password. There is intentionally no way to create a
/// JWT without first verifying the password.
pub fn create_jwt_if_valid_pw(
    user: &User,
    password: &str,
    secret: &str,
) -> Result<String, AuthError> {
    // Validate the password
    bcrypt::verify(password, &user.password_hash)
        .context("technical issue verifying password")?
        .then_some(())
        .ok_or(AuthError::InvalidPassword)?;

    jsonwebtoken::encode(
        &Header::default(),
        &Claims::new(user.id)?,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .context("failed to encode JWT")
    .map_err(Into::into)
}

/// Validates the JSON Web Token, returning the contained user ID if valid.
pub fn validate_jwt(token: &str, secret: &str) -> Result<i32, AuthError> {
    if let Ok(token_data) = jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    ) && let Ok(id) = token_data.claims.parse_sub()
    {
        Ok(id)
    } else {
        Err(AuthError::JwtValidation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Days, Utc};
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

    // Determined that testing hash_pw would be trivial

    #[test]
    fn encodes_and_decodes_id_for_valid_pw() {
        let correct_pw = "Bob's password";
        let secret = "shh_hhh_hhh";

        let bob = make_bob(correct_pw);
        let token = create_jwt_if_valid_pw(&bob, correct_pw, secret)
            .expect("failed to create JWT for valid password");

        let id = validate_jwt(&token, secret).expect("failed to validate token");
        assert_eq!(id, bob.id);
    }

    #[test]
    fn token_creation_errors_for_invalid_pw() {
        let bob = make_bob("correct password");
        let result = create_jwt_if_valid_pw(&bob, "incorrect password", "top secret");
        assert!(
            matches!(result, Err(AuthError::InvalidPassword)),
            "{}",
            format!("{result:?}").red(),
        );
    }

    #[test]
    fn validation_errors_for_invalid_token() {
        let password = "53cur1ty";
        let bob = make_bob(password);
        let secret = "no one's gonna know";

        let _ = create_jwt_if_valid_pw(&bob, password, secret).expect("failed to create token");
        assert!(matches!(
            validate_jwt("fake token", secret),
            Err(AuthError::JwtValidation),
        ));
    }

    #[test]
    fn validation_errors_for_invalid_secret() {
        let password = "pa$$ed654wood&24b1";
        let bob = make_bob(password);
        let secret = "shh";

        let token = create_jwt_if_valid_pw(&bob, password, secret).expect("failed to create token");
        assert!(matches!(
            validate_jwt(&token, "boo!"),
            Err(AuthError::JwtValidation)
        ));
    }
}
