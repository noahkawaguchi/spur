use crate::dto::responses::PostResponse;
use chrono::{DateTime, Utc};

/// The post entity as it exists in the database with the addition of the author's username.
#[cfg_attr(test, derive(Debug, Clone))]
pub struct PostWithAuthor {
    pub id: i32,
    pub author_id: Option<i32>,
    pub parent_id: Option<i32>,
    pub body: Option<String>,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
    pub archived_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    /// From the users table
    pub author_username: Option<String>,
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

#[cfg(test)]
mod post_info_test_impl {
    use super::*;
    use crate::test_utils::time::{both_none_or_within_one_second, within_one_second};

    impl PartialEq for PostWithAuthor {
        /// Performs standard equality checks for each field, except the time-based ones, for which
        /// two `DateTime`s are considered equal if they are within one second of each other.
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id
                && self.author_id == other.author_id
                && self.parent_id == other.parent_id
                && self.body == other.body
                && within_one_second(self.created_at, other.created_at)
                && both_none_or_within_one_second(self.edited_at, other.edited_at)
                && both_none_or_within_one_second(self.archived_at, other.archived_at)
                && both_none_or_within_one_second(self.deleted_at, other.deleted_at)
                && self.author_username == other.author_username
        }
    }
}
