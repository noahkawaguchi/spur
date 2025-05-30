use chrono::{DateTime, Utc};

pub struct Post {
    pub id: i32,
    pub author_id: i32,
    pub prompt_id: i32,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
}
