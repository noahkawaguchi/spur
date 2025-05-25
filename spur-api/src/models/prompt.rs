pub struct Prompt {
    pub id: i32,
    pub author_id: i32,
    pub body: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
