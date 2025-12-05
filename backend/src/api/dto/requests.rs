use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// A request for logging in to an existing account.
#[derive(Serialize, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[validate(email(message = "not a valid email address"))]
    pub email: String,

    #[validate(length(min = 1, message = "password cannot be empty"))]
    pub password: String,
}

/// A request for adding a friend by username.
#[derive(Serialize, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddFriendRequest {
    #[validate(length(min = 1, message = "recipient username cannot be empty"))]
    pub recipient_username: String,
}

/// A request for creating a new post.
#[derive(Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreatePostRequest {
    #[validate(range(min = 1, message = "parent ID must be positive"))]
    pub parent_id: i32,

    #[validate(length(min = 1, message = "post body cannot be empty"))]
    pub body: String,
}
