use crate::services::{
    domain_error::{AuthError, DomainError, FriendshipError},
    jwt_svc::JwtValidationError,
};
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
    #[error(transparent)]
    Request(#[from] validator::ValidationErrors),

    #[error("Expired or invalid token")]
    Token(#[from] JwtValidationError),

    #[error(transparent)]
    Domain(#[from] DomainError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::Request(_) => StatusCode::BAD_REQUEST,
            Self::Token(_) => StatusCode::UNAUTHORIZED,
            Self::Domain(e) => match e {
                DomainError::Auth(err) => match err {
                    AuthError::DuplicateEmail | AuthError::DuplicateUsername => {
                        StatusCode::CONFLICT
                    }
                    AuthError::InvalidEmail | AuthError::InvalidPassword => {
                        StatusCode::UNAUTHORIZED
                    }
                },
                DomainError::Friendship(err) => match err {
                    FriendshipError::NonexistentUser => StatusCode::NOT_FOUND,
                    FriendshipError::AlreadyFriends | FriendshipError::AlreadyRequested => {
                        StatusCode::CONFLICT
                    }
                },
                DomainError::Technical(_) => StatusCode::INTERNAL_SERVER_ERROR,
            },
        };

        let message = if let Self::Domain(DomainError::Technical(e)) = self {
            // TODO: use a logger
            eprintln!("{}", e.to_string().red());
            String::from("Internal server error")
        } else {
            self.to_string()
        };

        (status, Json(ErrorResponse { error: message })).into_response()
    }
}
