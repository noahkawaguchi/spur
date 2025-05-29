#[derive(serde::Serialize, serde::Deserialize)]
pub struct PromptWithAuthor {
    pub id: i32,
    pub author_username: String,
    pub body: String,
}
