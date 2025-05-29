use crate::password_validator::validate_struct_pw;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Validate)]
pub struct SignupRequest {
    #[validate(length(min = 1, message = "name cannot be empty"))]
    pub name: String,

    #[validate(email(message = "not a valid email address"))]
    pub email: String,

    #[validate(length(min = 1, message = "username cannot be empty"))]
    pub username: String,

    #[validate(custom(function = validate_struct_pw))]
    pub password: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "not a valid email address"))]
    pub email: String,

    #[validate(length(min = 1, message = "password cannot be empty"))]
    pub password: String,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct AddFriendRequest {
    #[validate(length(min = 1, message = "recipient username cannot be empty"))]
    pub recipient_username: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetFriendsParam {
    pub pending: bool,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct CreatePromptRequest {
    #[validate(length(min = 1, message = "prompt body cannot be empty"))]
    pub body: String,
}

#[derive(Serialize, Deserialize)]
pub struct PromptsByAuthorParam {
    pub author_username: Option<String>,
}
