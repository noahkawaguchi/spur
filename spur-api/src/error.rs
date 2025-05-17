use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colored::Colorize;
use spur_shared::responses::ErrorResponse;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("{0}")]
    FieldValidation(#[from] validator::ValidationErrors),

    #[error("{0}")]
    Credentials(String),

    #[error("Expired or invalid token")]
    Token,

    #[error("{0}")]
    Nonexistent(String),

    #[error("{0}")]
    Duplicate(String),

    #[error(transparent)]
    Technical(#[from] TechnicalError),
}

#[derive(Debug, Error)]
pub enum TechnicalError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),

    #[error(transparent)]
    Bcrypt(#[from] bcrypt::BcryptError),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            Self::Technical(err) => {
                // TODO: use a logger
                match err {
                    TechnicalError::Database(e) => {
                        eprintln!("{}", format!("Database error: {e}").red());
                    }
                    TechnicalError::Bcrypt(e) => {
                        eprintln!("{}", format!("Bcrypt error: {e}").red());
                    }
                    TechnicalError::Other(e) => {
                        eprintln!("{}", format!("Technical error: {e}").red());
                    }
                }

                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse { error: String::from("Internal server error") }),
                )
                    .into_response()
            }

            Self::FieldValidation(e) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse { error: e.to_string() }),
            )
                .into_response(),

            Self::Credentials(error) => {
                (StatusCode::UNAUTHORIZED, Json(ErrorResponse { error })).into_response()
            }

            Self::Token => (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse { error: String::from("Expired or invalid token") }),
            )
                .into_response(),

            Self::Nonexistent(error) => {
                (StatusCode::NOT_FOUND, Json(ErrorResponse { error })).into_response()
            }

            Self::Duplicate(error) => {
                (StatusCode::CONFLICT, Json(ErrorResponse { error })).into_response()
            }
        }
    }
}
