use chrono::{DateTime, Utc};
use spur_shared::models::PromptWithAuthor;

#[cfg_attr(test, derive(Debug, PartialEq, Eq, Clone))]
pub struct PromptInfo {
    pub id: i32,
    pub author_id: i32,
    pub author_username: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

impl From<PromptInfo> for PromptWithAuthor {
    fn from(info: PromptInfo) -> Self {
        Self { id: info.id, author_username: info.author_username, body: info.body }
    }
}
