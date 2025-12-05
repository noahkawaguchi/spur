use crate::models::post::PostWithAuthor;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// A general-purpose error response.
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// A general-purpose success response.
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
#[derive(Serialize, Deserialize, ToSchema)]
pub struct SuccessResponse {
    pub message: String,
}

/// A response for sending an auth token.
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
#[derive(Serialize, Deserialize, ToSchema)]
pub struct TokenResponse {
    pub token: String,
}

/// A response for sending information about a post.
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PostResponse {
    /// The numeric ID of the post.
    pub id: i32,
    /// The username of the author of the post.
    pub author_username: String,
    /// The ID of the post that this post is in reply to.
    pub parent_id: Option<i32>,
    /// The content of the post.
    pub body: String,
    /// The time the post was created in milliseconds since the Unix epoch.
    pub created_at_ms: i64,
    /// If edited, the time the post was edited in milliseconds since the Unix epoch.
    pub edited_at_ms: Option<i64>,
    /// If archived, the time the post was archived in milliseconds since the Unix epoch.
    pub archived_at_ms: Option<i64>,
    /// If deleted, the time the post was deleted in milliseconds since the Unix epoch.
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
