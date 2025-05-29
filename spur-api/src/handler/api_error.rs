use crate::domain::{
    error::DomainError, friendship::error::FriendshipError, prompt::PromptError, user::UserError,
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

    #[error(transparent)]
    Domain(#[from] DomainError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::Request(_) => StatusCode::BAD_REQUEST,
            Self::Domain(e) => match e {
                DomainError::Auth(_) => StatusCode::UNAUTHORIZED,
                DomainError::User(err) => match err {
                    UserError::NotFound => StatusCode::NOT_FOUND,
                    UserError::DuplicateEmail | UserError::DuplicateUsername => {
                        StatusCode::CONFLICT
                    }
                },
                DomainError::Friendship(err) => match err {
                    FriendshipError::SelfFriendship => StatusCode::BAD_REQUEST,
                    FriendshipError::NonexistentUser => StatusCode::NOT_FOUND,
                    FriendshipError::AlreadyFriends | FriendshipError::AlreadyRequested => {
                        StatusCode::CONFLICT
                    }
                },
                DomainError::Prompt(err) => match err {
                    PromptError::Duplicate => StatusCode::CONFLICT,
                    PromptError::NotFound => StatusCode::NOT_FOUND,
                    PromptError::NotFriends => StatusCode::FORBIDDEN,
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
