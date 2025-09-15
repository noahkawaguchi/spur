use crate::dto::password_validator::validate_struct_pw;
use lazy_regex::{Regex, lazy_regex};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use validator::Validate;

static USERNAME_RE: LazyLock<Regex> = LazyLock::new(|| lazy_regex!("^[A-Za-z0-9_-]+$").clone());

#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
#[derive(Serialize, Deserialize, Validate)]
pub struct SignupRequest {
    #[validate(length(min = 1, message = "name cannot be empty"))]
    pub name: String,

    #[validate(email(message = "not a valid email address"))]
    pub email: String,

    #[validate(regex(
        path = *USERNAME_RE,
        message = "username may only contain ASCII letters, numbers, underscores, and hyphens",
    ))]
    pub username: String,

    #[validate(custom(function = validate_struct_pw))]
    pub password: String,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "not a valid email address"))]
    pub email: String,

    #[validate(length(min = 1, message = "password cannot be empty"))]
    pub password: String,
}

#[derive(Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct AddFriendRequest {
    #[validate(length(min = 1, message = "recipient username cannot be empty"))]
    pub recipient_username: String,
}

#[derive(Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreatePostRequest {
    #[validate(range(min = 1, message = "parent ID must be positive"))]
    pub parent_id: i32,

    #[validate(length(min = 1, message = "post body cannot be empty"))]
    pub body: String,
}
