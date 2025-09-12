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
