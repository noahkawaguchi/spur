pub mod auth;
pub mod friendship;
pub mod post;
pub mod user;

#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    #[error("Unique constraint violation: {0}")]
    UniqueViolation(String),

    #[error("Check constraint violation: {0}")]
    CheckViolation(String),

    #[error("Technical error: {0}")]
    Technical(#[from] anyhow::Error),
}

#[cfg(test)]
impl Clone for RepoError {
    /// Clones `self` as expected, except for the `Technical` variant, for which a new
    /// `anyhow::Error` is created from the string representation of the existing one.
    fn clone(&self) -> Self {
        use anyhow::anyhow;

        match self {
            Self::UniqueViolation(s) => Self::UniqueViolation(s.clone()),
            Self::CheckViolation(s) => Self::CheckViolation(s.clone()),
            Self::Technical(e) => Self::Technical(anyhow!(e.to_string())),
        }
    }
}
