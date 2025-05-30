#[derive(Debug, thiserror::Error)]
pub enum ContentError {
    #[error("You have already created the same prompt")]
    DuplicatePrompt,

    #[error("You cannot write a post in response to your own prompt")]
    OwnPrompt,

    #[error("No content found")]
    NotFound,

    #[error("You must be friends to see someone's content")]
    NotFriends,
}
