#![allow(clippy::unnecessary_wraps)] // Because inquire expects a `Result` from validators

use email_address::EmailAddress;
use inquire::{
    CustomUserError,
    validator::{ErrorMessage, Validation},
};
use spur_shared::password_validator::validate_password;

pub fn email(input: &str) -> Result<Validation, CustomUserError> {
    if EmailAddress::is_valid(input) {
        Ok(Validation::Valid)
    } else {
        Ok(Validation::Invalid(ErrorMessage::Custom(String::from(
            "not a valid email address",
        ))))
    }
}

pub fn password(input: &str) -> Result<Validation, CustomUserError> {
    match validate_password(input) {
        Ok(()) => Ok(Validation::Valid),
        Err(e) => Ok(Validation::Invalid(ErrorMessage::Custom(e))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod email {
        use super::*;

        #[test]
        fn returns_valid_for_valid_email() {
            assert_eq!(
                email("hello@email.com").expect("should never be Err"),
                Validation::Valid,
            );
        }

        #[test]
        fn returns_invalid_for_invalid_email() {
            // NOTE: not testing the email validation function itself here
            assert_eq!(
                email("not an email").expect("should never be Err"),
                Validation::Invalid(ErrorMessage::Custom(String::from(
                    "not a valid email address",
                ))),
            );
        }
    }

    mod password {
        use super::*;

        #[test]
        fn returns_valid_for_valid_password() {
            assert_eq!(
                password("qwertyQWERTY789!@#").expect("should never be Err"),
                Validation::Valid,
            );
        }

        #[test]
        fn returns_invalid_for_invalid_password() {
            // NOTE: not testing the password validation function itself here
            assert_eq!(
                password("abc").expect("should never be Err"),
                Validation::Invalid(ErrorMessage::Custom(String::from(
                    "password must be at least 10 characters"
                ))),
            );
        }
    }
}
