use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct PromptWithAuthor {
    pub id: i32,
    pub author_username: String,
    pub body: String,
}

impl fmt::Display for PromptWithAuthor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r#"Prompt {} (by {}): "{}""#,
            self.id, self.author_username, self.body
        )
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct PostWithPrompt {
    pub id: i32,
    pub author_username: String,
    pub prompt: PromptWithAuthor,
    pub body: String,
}

impl fmt::Display for PostWithPrompt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Post {} (by {}) in response to:\n  {}\n\n{}\n\n",
            self.id, self.author_username, self.prompt, self.body,
        )
    }
}
