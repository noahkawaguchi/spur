use chrono::{DateTime, Utc};
use spur_shared::models::{PostWithPrompt, PromptWithAuthor};

pub struct Post {
    pub id: i32,
    pub author_id: i32,
    pub prompt_id: i32,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
}

pub struct PostWithPromptRow {
    pub post_id: i32,
    pub post_author_username: String,
    pub post_body: String,
    pub prompt_id: i32,
    pub prompt_author_username: String,
    pub prompt_body: String,
}

impl From<PostWithPromptRow> for PostWithPrompt {
    fn from(row: PostWithPromptRow) -> Self {
        Self {
            id: row.post_id,
            author_username: row.post_author_username,
            prompt: PromptWithAuthor {
                id: row.prompt_id,
                author_username: row.prompt_author_username,
                body: row.prompt_body,
            },
            body: row.post_body,
        }
    }
}
