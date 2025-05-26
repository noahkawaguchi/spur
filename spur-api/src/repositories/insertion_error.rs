use sqlx::postgres::PgDatabaseError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InsertionError {
    #[error("Technical database error: {0}")]
    Technical(sqlx::Error),

    #[error("Unique constraint violation error: {0}")]
    UniqueViolation(String),
}

impl From<sqlx::Error> for InsertionError {
    fn from(err: sqlx::Error) -> Self {
        if let sqlx::Error::Database(db_err) = &err {
            if let Some(violation) = db_err
                .try_downcast_ref::<PgDatabaseError>()
                .filter(|pg| pg.code() == "23505") // PG error code for unique violation
                .map(|unique| unique.constraint().unwrap_or("<unnamed unique constraint>"))
            {
                return Self::UniqueViolation(violation.to_string());
            }
        }

        Self::Technical(err)
    }
}
