#[derive(Debug, thiserror::Error)]
pub enum ReadError {
    #[error(transparent)]
    Technical(#[from] anyhow::Error),
}

impl From<sqlx::Error> for ReadError {
    fn from(e: sqlx::Error) -> Self { anyhow::Error::from(e).into() }
}

#[async_trait::async_trait]
pub trait SocialRead: Send + Sync {
    /// Retrieves the usernames of all confirmed friends of the user with the provided ID.
    async fn friend_usernames(&self, id: i32) -> Result<Vec<String>, ReadError>;

    /// Retrieves the usernames of all users who have pending requests to the user with the
    /// provided ID.
    async fn pending_requests(&self, id: i32) -> Result<Vec<String>, ReadError>;
}

pub trait PostWithAuthorRead {}
