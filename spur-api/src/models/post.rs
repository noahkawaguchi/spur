use chrono::{DateTime, Utc};
use spur_shared::models::{PostWithPrompt, PromptWithAuthor};

#[cfg_attr(test, derive(Debug, PartialEq, Eq, Clone))]
pub struct PostInfo {
    pub id: i32,
    pub author_id: i32,
    pub author_username: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,

    pub prompt_id: i32,
    pub prompt_author_username: String,
    pub prompt_body: String,
}

impl From<PostInfo> for PostWithPrompt {
    fn from(info: PostInfo) -> Self {
        Self {
            id: info.id,
            author_username: info.author_username,
            prompt: PromptWithAuthor {
                id: info.prompt_id,
                author_username: info.prompt_author_username,
                body: info.prompt_body,
            },
            body: info.body,
        }
    }
}
