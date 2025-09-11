use crate::read::ReadError;

pub mod post_with_author_read;
pub mod social_read;

impl From<sqlx::Error> for ReadError {
    fn from(e: sqlx::Error) -> Self { anyhow::Error::from(e).into() }
}
