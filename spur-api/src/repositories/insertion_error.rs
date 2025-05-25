use sqlx::postgres::PgDatabaseError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InsertionError {
    #[error("Technical database error: {0}")]
    Technical(#[from] sqlx::Error),

    #[error("Unique constraint violation error")]
    UniqueViolation,
}

pub trait SqlxErrExt {
    /// Checks whether or not an `sqlx::Error` was due to a violation of a UNIQUE constraint.
    fn is_unique_violation(&self) -> bool;
}

impl SqlxErrExt for sqlx::Error {
    fn is_unique_violation(&self) -> bool {
        match self {
            Self::Database(db_err) => db_err
                .try_downcast_ref::<PgDatabaseError>()
                .is_some_and(|e| e.code() == "23505"),
            _ => false,
        }
    }
}
