use crate::domain::RepoError;
use anyhow::anyhow;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PostError {
    #[error("No post found")]
    NotFound,
    // TODO: implement editing posts
    #[error("Cannot reply multiple times to the same post. Try editing the existing reply.")]
    DuplicateReply,

    #[error("Cannot reply to a deleted post")]
    DeletedParent,

    #[error("Cannot reply to an archived post")]
    ArchivedParent,

    #[error("Cannot reply to one's own post")]
    SelfReply,

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl From<RepoError> for PostError {
    fn from(e: RepoError) -> Self {
        match e {
            RepoError::UniqueViolation(v) if v == "post_author_parent_unique" => {
                Self::DuplicateReply
            }
            RepoError::UniqueViolation(v) => {
                Self::Internal(anyhow!("Unexpected unique violation: {v}"))
            }
            RepoError::CheckViolation(v) if v == "text_non_empty" => {
                Self::Internal(anyhow!("Empty field made it past request validation: {v}"))
            }
            RepoError::CheckViolation(v) => {
                Self::Internal(anyhow!("Unexpected check violation: {v}"))
            }
            RepoError::Technical(e) => Self::Internal(e),
        }
    }
}

#[cfg(test)]
impl PartialEq for PostError {
    /// Compares the string representation of `e` for `Internal(e)`. Otherwise, just checks that the
    /// variant is the same.
    fn eq(&self, other: &Self) -> bool {
        use std::mem::discriminant;

        match self {
            Self::Internal(self_e) => {
                matches!(other,
                    Self::Internal(other_e) if self_e.to_string() == other_e.to_string())
            }
            _ => discriminant(self) == discriminant(other),
        }
    }
}
