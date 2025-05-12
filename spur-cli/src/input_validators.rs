use email_address::EmailAddress;
use inquire::{
    CustomUserError,
    validator::{ErrorMessage, Validation},
};
use spur_shared::validator::validate_password;

#[allow(clippy::unnecessary_wraps)] // Because inquire expects a Result
pub fn nonempty(input: &str) -> Result<Validation, CustomUserError> {
    if input.is_empty() {
        Ok(Validation::Invalid(ErrorMessage::Custom(String::from(
            "cannot be empty",
        ))))
    } else {
        Ok(Validation::Valid)
    }
}

#[allow(clippy::unnecessary_wraps)] // Because inquire expects a Result
pub fn email(input: &str) -> Result<Validation, CustomUserError> {
    if EmailAddress::is_valid(input) {
        Ok(Validation::Valid)
    } else {
        Ok(Validation::Invalid(ErrorMessage::Custom(String::from(
            "not a valid email address",
        ))))
    }
}

#[allow(clippy::unnecessary_wraps)] // Because inquire expects a Result
pub fn password(input: &str) -> Result<Validation, CustomUserError> {
    match validate_password(input) {
        Ok(()) => Ok(Validation::Valid),
        Err(e) => Ok(Validation::Invalid(ErrorMessage::Custom(e))),
    }
}
