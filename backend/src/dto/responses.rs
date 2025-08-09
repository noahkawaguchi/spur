use crate::models::post::PostResponse;
use serde::{Deserialize, Serialize};

/// A general-purpose error response body.
#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// A general-purpose success response body.
#[derive(Serialize, Deserialize)]
pub struct SuccessResponse {
    pub message: String,
}

/// A response body for sending a JWT.
#[derive(Serialize, Deserialize)]
pub struct TokenResponse {
    pub token: String,
}

/// A response body for listing the usernames of a set of users.
#[derive(Serialize, Deserialize)]
pub struct UsernamesResponse {
    pub usernames: Vec<String>,
}

/// A response body for sending one post.
#[derive(Serialize, Deserialize)]
pub struct SinglePostResponse {
    pub post: PostResponse,
}

/// A response body for sending many posts.
#[derive(Serialize, Deserialize)]
pub struct ManyPostsResponse {
    pub posts: Vec<PostResponse>,
}
