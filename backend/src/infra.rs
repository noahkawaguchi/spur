use crate::{domain::RepoError, read_models::ReadError};

pub mod friendship_repo;
pub mod post_repo;
pub mod post_with_author_read;
pub mod social_read;
pub mod user_repo;

impl From<sqlx::Error> for ReadError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => Self::NotFound,
            _ => anyhow::Error::from(e).into(),
        }
    }
}

impl From<sqlx::Error> for RepoError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::Database(ref db_err) => match db_err.code().as_deref() {
                Some("23505") => Self::UniqueViolation(
                    db_err
                        .constraint()
                        .unwrap_or("<unnamed unique constraint>")
                        .to_string(),
                ),
                Some("23514") => Self::CheckViolation(
                    db_err
                        .constraint()
                        .unwrap_or("<unnamed check constraint>")
                        .to_string(),
                ),
                _ => Self::Technical(e.into()),
            },
            _ => Self::Technical(e.into()),
        }
    }
}
