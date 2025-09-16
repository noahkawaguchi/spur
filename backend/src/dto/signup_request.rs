use lazy_regex::{Regex, lazy_regex};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use validator::{Validate, ValidationError};

static USERNAME_RE: LazyLock<Regex> = LazyLock::new(|| lazy_regex!("^[A-Za-z0-9_-]+$").clone());
const LENGTH_CODE: &str = "length";
const CHARS_CODE: &str = "character_classes";

#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
#[derive(Serialize, Deserialize, Validate)]
pub struct SignupRequest {
    #[validate(length(min = 1, message = "name cannot be empty"))]
    pub name: String,

    #[validate(email(message = "not a valid email address"))]
    pub email: String,

    #[validate(regex(
        path = *USERNAME_RE,
        message = "username may only contain English letters, digits, underscores, and hyphens",
    ))]
    pub username: String,

    #[validate(custom(function = validate_password))]
    pub password: String,
}

/// # Errors
///
/// Will return `Err` if `password` does not meet the following requirements:
///
/// - At least 10 characters
/// - At most 72 bytes
/// - At least one lowercase letter, uppercase letter, digit, and special character
fn validate_password(password: &str) -> Result<(), ValidationError> {
    if password.chars().count() < 10 {
        return Err(ValidationError::new(LENGTH_CODE)
            .with_message("password must be at least 10 characters".into()));
    }

    // Because 72 bytes is the max length for bcrypt
    if password.len() > 72 {
        return Err(ValidationError::new(LENGTH_CODE).with_message(
            "password must be at most 72 bytes \
                (72 English letters/digits/symbols, fewer for other kinds of characters)"
                .into(),
        ));
    }

    if !password.chars().any(char::is_lowercase) {
        return Err(ValidationError::new(CHARS_CODE)
            .with_message("password must contain at least one lowercase letter".into()));
    }

    if !password.chars().any(char::is_uppercase) {
        return Err(ValidationError::new(CHARS_CODE)
            .with_message("password must contain at least one uppercase letter".into()));
    }

    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err(ValidationError::new(CHARS_CODE)
            .with_message("password must contain at least one digit (0-9)".into()));
    }

    if !password.chars().any(|c| !c.is_alphanumeric()) {
        return Err(ValidationError::new(CHARS_CODE)
            .with_message("password must contain at least one special character".into()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_valid_usernames() {
        for username in [
            "John",
            "ri_cha_ko",
            "Heavenly-Sounds",
            "27trombones",
            "yeah-_-oh",
            "ZOOM500",
        ] {
            assert!(
                SignupRequest {
                    name: String::from("John"),
                    email: String::from("john@email.mail"),
                    username: username.to_string(),
                    password: String::from("ok72PAss!yEAH()Pa55"),
                }
                .validate()
                .is_ok()
            );
        }
    }

    #[test]
    fn disallows_invalid_usernames() {
        for username in [
            "ジョンだにょーん",
            "my２s　７",
            "hello💄💄💄world",
            "",
            "           ",
            "cool+person",
            "free(p); free(p);",
        ] {
            assert!(
                SignupRequest {
                    name: String::from("John"),
                    email: String::from("john@email.mail"),
                    username: username.to_string(),
                    password: String::from("ok72PAss!yEAH()Pa55"),
                }
                .validate()
                .is_err_and(|e| e.to_string()
                    == "username: username may only contain \
                    English letters, digits, underscores, and hyphens")
            );
        }
    }

    #[test]
    fn allows_valid_passwords() {
        for password in [
            "secret1SECRET2$#",
            "#hash$dollar$MONEY21",
            "Three*Men && 4_cars",
            "C0rr3ct H0r$3 B4tt3ry 5t5p13",
        ] {
            assert!(
                SignupRequest {
                    name: String::from("John"),
                    email: String::from("john@email.mail"),
                    username: String::from("john-is-cool"),
                    password: password.to_string(),
                }
                .validate()
                .is_ok()
            );
        }
    }

    #[test]
    fn disallows_invalid_passwords() {
        for [password, msg] in [
            ["", "be at least 10 characters"],
            ["  ", "be at least 10 characters"],
            ["aB4%", "be at least 10 characters"],
            [
                "aaaaaaaaaaBBBBBBBBBB4444444444()()()()()UUUUUUUUUU-hhhhhhhhhhEEEEEEEEEE8888888888",
                "be at most 72 bytes (72 English letters/digits/symbols, \
                    fewer for other kinds of characters)",
            ],
            [
                "                           ",
                "contain at least one lowercase letter",
            ],
            [
                "abba--babb-bdbe-bo-gu-ugo-ao-bl",
                "contain at least one uppercase letter",
            ],
            ["abOU_ID_FG basicKFG", "contain at least one digit (0-9)"],
            ["abc123ABC123abc9", "contain at least one special character"],
        ] {
            assert!(
                SignupRequest {
                    name: String::from("John"),
                    email: String::from("john@email.mail"),
                    username: String::from("john-is-cool"),
                    password: password.to_string(),
                }
                .validate()
                .is_err_and(|e| e.to_string() == format!("password: password must {msg}")),
            );
        }
    }
}
