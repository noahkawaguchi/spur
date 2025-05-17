use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colored::Colorize;
use spur_shared::responses::ErrorResponse;
use thiserror::Error;

#[derive(Debug, Error)]
enum ApiError {
    #[error(transparent)]
    Technical(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            Self::Technical(e) => {
                eprintln!("{}", e.to_string().red()); // TODO: use a logger
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse { error: String::from("Internal server error") }),
                )
                    .into_response()
            }
        }
    }
}
