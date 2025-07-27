use crate::{
    domain::{
        auth::{AuthError, Claims},
        error::DomainError,
    },
    models::user::{NewUser, User, UserRegistration},
    technical_error::TechnicalError,
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};

pub fn hash_pw(reg: UserRegistration) -> Result<NewUser, DomainError> {
    let pw_hash =
        bcrypt::hash(&reg.password, bcrypt::DEFAULT_COST).map_err(TechnicalError::from)?;

    Ok(reg.into_new_user_with(pw_hash))
}

pub fn create_jwt(id: i32, secret: &str) -> Result<String, DomainError> {
    jsonwebtoken::encode(
        &Header::default(),
        &Claims::new(id)?,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|e| TechnicalError::from(e).into())
}

pub fn create_jwt_if_valid_pw(
    user: &User,
    password: &str,
    secret: &str,
) -> Result<String, DomainError> {
    // Validate the password
    bcrypt::verify(password, &user.password_hash)
        .map_err(TechnicalError::from)?
        .then_some(())
        .ok_or(AuthError::InvalidPassword)?;

    create_jwt(user.id, secret)
}

pub fn validate_jwt(token: &str, secret: &str) -> Result<i32, DomainError> {
    if let Ok(token_data) = jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    ) {
        if let Ok(id) = token_data.claims.parse_sub() {
            return Ok(id);
        }
    }

    Err(AuthError::JwtValidation.into())
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

    // Determined that testing hash_pw would be trivial and that create_jwt is sufficiently tested
    // via create_jwt_if_valid_pw

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

        let _ = create_jwt_if_valid_pw(&bob, password, secret).expect("failed to create token");
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

        let token = create_jwt_if_valid_pw(&bob, password, secret).expect("failed to create token");
        assert!(matches!(
            validate_jwt(&token, "boo!"),
            Err(DomainError::Auth(AuthError::JwtValidation)),
        ));
    }
}
