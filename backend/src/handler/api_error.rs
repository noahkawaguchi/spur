use crate::{
    domain::{
        error::DomainError, friendship::error::FriendshipError, post::PostError, user::UserError,
    },
    dto::responses::ErrorResponse,
};
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colored::Colorize;
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
            Self::Request(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Domain(e) => match e {
                DomainError::Auth(_) => StatusCode::UNAUTHORIZED,
                DomainError::User(err) => match err {
                    UserError::NotFound => StatusCode::NOT_FOUND,
                    UserError::DuplicateEmail | UserError::DuplicateUsername => {
                        StatusCode::CONFLICT
                    }
                },
                DomainError::Friendship(err) => match err {
                    FriendshipError::SelfFriendship => StatusCode::UNPROCESSABLE_ENTITY,
                    FriendshipError::NonexistentUser => StatusCode::NOT_FOUND,
                    FriendshipError::AlreadyFriends | FriendshipError::AlreadyRequested => {
                        StatusCode::CONFLICT
                    }
                },
                DomainError::Post(err) => match err {
                    PostError::NotFound => StatusCode::NOT_FOUND,
                    PostError::DuplicateReply => StatusCode::CONFLICT,
                    PostError::DeletedParent => StatusCode::GONE,
                    PostError::SelfReply | PostError::ArchivedParent => {
                        StatusCode::UNPROCESSABLE_ENTITY
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
