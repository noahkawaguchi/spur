#[derive(serde::Serialize, serde::Deserialize)]
pub struct PromptWithAuthor {
    pub id: i32,
    pub author_username: String,
    pub body: String,
}

impl std::fmt::Display for PromptWithAuthor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"Prompt {} (by {}): "{}""#,
            self.id, self.author_username, self.body
        )
    }
}
