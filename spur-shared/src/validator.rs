use crate::dto::{LoginRequest, SignupRequest};
use email_address::EmailAddress;

/// Confirms that `name` and `username` are non-empty, `email` is in a valid email address format,
/// and `password` meets the following requirements:
///
/// - At least 10 characters
/// - At most 72 bytes
/// - At least one lowercase letter, uppercase letter, digit, and special character
pub fn validate_signup_request(req: &SignupRequest) -> Result<(), String> {
    if req.name.is_empty() {
        return Err(String::from("name cannot be empty"));
    }

    if req.username.is_empty() {
        return Err(String::from("username cannot be empty"));
    }

    if !EmailAddress::is_valid(&req.email) {
        return Err(String::from("not a valid email address"));
    }

    validate_password(&req.password)
}

/// Confirms that `email` is in a valid email address format and `password` is non-empty. Does not
/// check specific password requirements because a new password is not being created.
pub fn validate_login_request(req: &LoginRequest) -> Result<(), String> {
    if !EmailAddress::is_valid(&req.email) {
        return Err(String::from("not a valid email address"));
    }

    if req.password.is_empty() {
        return Err(String::from("password cannot be empty"));
    }

    Ok(())
}

fn validate_password(password: &str) -> Result<(), String> {
    if password.chars().count() < 10 {
        return Err(String::from("password must be at least 10 characters"));
    }

    if password.len() > 72 {
        return Err(String::from("password must not be more than 72 bytes"));
    }

    if !password.chars().any(|c| c.is_ascii_uppercase()) {
        return Err(String::from(
            "password must contain at least one uppercase letter",
        ));
    }

    if !password.chars().any(|c| c.is_ascii_lowercase()) {
        return Err(String::from(
            "password must contain at least one lowercase letter",
        ));
    }

    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err(String::from("password must contain at least one digit"));
    }

    if !password.chars().any(|c| !c.is_alphanumeric()) {
        return Err(String::from(
            "password must contain at least one special character",
        ));
    }

    Ok(())
}
