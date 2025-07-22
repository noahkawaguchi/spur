use validator::ValidationError;

/// # Errors
///
/// Will return `Err` if `password` does not meet the following requirements:
///
/// - At least 10 characters
/// - At most 72 bytes
/// - At least one lowercase letter, uppercase letter, digit, and special character
pub fn validate_struct_pw(password: &str) -> Result<(), ValidationError> {
    validate_password(password).map_err(|e| ValidationError::new("").with_message(e.into()))
}

/// # Errors
///
/// Will return `Err` if `password` does not meet the following requirements:
///
/// - At least 10 characters
/// - At most 72 bytes
/// - At least one lowercase letter, uppercase letter, digit, and special character
pub fn validate_password(password: &str) -> Result<(), String> {
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
