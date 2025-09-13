use serde::{Deserialize, Serialize};

/// A general-purpose error response struct.
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// A general-purpose success response struct.
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
#[derive(Serialize, Deserialize)]
pub struct SuccessResponse {
    pub message: String,
}

/// A response struct for sending a JWT.
#[derive(Serialize, Deserialize)]
pub struct TokenResponse {
    pub token: String,
}

/// A response struct for sending information about a post.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostResponse {
    pub id: i32,
    pub author_username: String,
    pub parent_id: Option<i32>,
    pub body: String,
    pub created_at_ms: i64,
    pub edited_at_ms: Option<i64>,
    pub archived_at_ms: Option<i64>,
    pub deleted_at_ms: Option<i64>,
}
