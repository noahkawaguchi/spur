use serde::{Deserialize, Serialize};

use crate::models::PromptWithAuthor;

/// A general-purpose error response body.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// A general-purpose success response body.
#[derive(Serialize, Deserialize)]
pub struct SuccessResponse {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
}

/// A response body for listing the usernames of a set of users.
#[derive(Serialize, Deserialize)]
pub struct UsernamesResponse {
    pub usernames: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct CreatePromptResponse {
    pub prompt_id: i32,
}

#[derive(Serialize, Deserialize)]
pub struct SinglePromptResponse {
    pub prompt: PromptWithAuthor,
}

#[derive(Serialize, Deserialize)]
pub struct MultiplePromptsResponse {
    pub prompts: Vec<PromptWithAuthor>,
}
