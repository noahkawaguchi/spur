use crate::models::post::PostWithAuthor;
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

/// A response struct for sending an auth token.
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
#[derive(Serialize, Deserialize)]
pub struct TokenResponse {
    pub token: String,
}

/// A response struct for sending information about a post.
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
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

impl From<PostWithAuthor> for PostResponse {
    fn from(pwa: PostWithAuthor) -> Self {
        Self {
            id: pwa.id,
            author_username: pwa
                .author_username
                .unwrap_or_else(|| String::from("[deleted]")),
            parent_id: pwa.parent_id,
            body: pwa.body.unwrap_or_else(|| String::from("[deleted]")),
            created_at_ms: pwa.created_at.timestamp_millis(),
            edited_at_ms: pwa.edited_at.map(|ms| ms.timestamp_millis()),
            archived_at_ms: pwa.archived_at.map(|ms| ms.timestamp_millis()),
            deleted_at_ms: pwa.deleted_at.map(|ms| ms.timestamp_millis()),
        }
    }
}
