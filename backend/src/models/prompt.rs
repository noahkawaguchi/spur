use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg_attr(test, derive(Debug, PartialEq, Eq, Clone))]
pub struct PromptInfo {
    pub id: i32,
    pub author_id: i32,
    pub author_username: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

#[cfg_attr(test, derive(Debug, PartialEq, Eq, Clone))]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptWithAuthor {
    pub id: i32,
    pub author_username: String,
    pub body: String,
}

impl From<PromptInfo> for PromptWithAuthor {
    fn from(info: PromptInfo) -> Self {
        Self { id: info.id, author_username: info.author_username, body: info.body }
    }
}
