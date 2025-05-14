#![allow(clippy::unnecessary_wraps)] // Because inquire expects a `Result` from validators

use email_address::EmailAddress;
use inquire::{
    CustomUserError,
    validator::{ErrorMessage, Validation},
};
use spur_shared::validator::validate_password;

pub fn nonempty(input: &str) -> Result<Validation, CustomUserError> {
    if input.is_empty() {
        Ok(Validation::Invalid(ErrorMessage::Custom(String::from(
            "cannot be empty",
        ))))
    } else {
        Ok(Validation::Valid)
    }
}

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
