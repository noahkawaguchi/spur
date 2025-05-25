use sqlx::postgres::PgDatabaseError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InsertionError {
    #[error("Technical database error: {0}")]
    Technical(#[from] sqlx::Error),

    #[error("Unique constraint violation error: {0}")]
    UniqueViolation(String),
}

pub trait SqlxErrExt {
    /// Gets the specific constraint name or `"<unnamed unique constraint>"` if an `sqlx::Error`
    /// was due to a violation of a UNIQUE constraint. Returns None for other types of errors.
    fn unique_violation(&self) -> Option<String>;
}

impl SqlxErrExt for sqlx::Error {
    fn unique_violation(&self) -> Option<String> {
        match self {
            Self::Database(pg_err) => pg_err
                .try_downcast_ref::<PgDatabaseError>()
                .filter(|e| e.code() == "23505")
                .map(|e| {
                    e.constraint()
                        .unwrap_or("<unnamed unique constraint>")
                        .to_string()
                }),
            _ => None,
        }
    }
}
