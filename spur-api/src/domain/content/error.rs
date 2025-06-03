#[derive(Debug, thiserror::Error)]
pub enum ContentError {
    #[error("Already created the same prompt")]
    DuplicatePrompt,

    #[error(
        "Cannot create multiple posts in response to the same prompt. \
        Try editing the existing post." // TODO: implement editing posts
    )]
    DuplicatePost,

    #[error("Cannot write a post in response to one's own prompt")]
    OwnPrompt,

    #[error("Content not found")]
    NotFound,

    #[error("Must be friends to see someone's content")]
    NotFriends,
}
