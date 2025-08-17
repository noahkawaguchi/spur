use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepoError {
    #[error("Unique constraint violation: {0}")]
    UniqueViolation(String),

    #[error("Check constraint violation: {0}")]
    CheckViolation(String),

    #[error("Technical database error: {0}")]
    Technical(sqlx::Error),

    #[error("Unexpected error: {0}")]
    Unexpected(#[from] anyhow::Error),
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
                _ => Self::Technical(e),
            },
            _ => Self::Technical(e),
        }
    }
}
