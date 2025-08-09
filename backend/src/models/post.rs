use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The post entity as it exists in the database with the addition of the author's username.
pub struct PostInfo {
    pub id: i32,
    pub author_id: Option<i32>,
    pub parent_id: Option<i32>,
    pub body: Option<String>,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
    pub archived_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,

    pub author_username: Option<String>,
}

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

impl From<PostInfo> for PostResponse {
    fn from(info: PostInfo) -> Self {
        Self {
            id: info.id,
            author_username: info
                .author_username
                .unwrap_or_else(|| String::from("[deleted]")),
            parent_id: info.parent_id,
            body: info.body.unwrap_or_else(|| String::from("[deleted]")),
            created_at_ms: info.created_at.timestamp_millis(),
            edited_at_ms: info.edited_at.map(|ms| ms.timestamp_millis()),
            archived_at_ms: info.archived_at.map(|ms| ms.timestamp_millis()),
            deleted_at_ms: info.deleted_at.map(|ms| ms.timestamp_millis()),
        }
    }
}
