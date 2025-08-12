use crate::{
    domain::{
        auth::AuthError, friendship::error::FriendshipError, post::PostError, user::UserError,
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
    Auth(#[from] AuthError),
    #[error(transparent)]
    User(#[from] UserError),
    #[error(transparent)]
    Friendship(#[from] FriendshipError),
    #[error(transparent)]
    Post(#[from] PostError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::Request(_)
            | Self::Friendship(FriendshipError::SelfFriendship)
            | Self::Post(PostError::SelfReply | PostError::ArchivedParent) => {
                (StatusCode::UNPROCESSABLE_ENTITY, self.to_string())
            }

            Self::Auth(AuthError::JwtValidation | AuthError::InvalidPassword) => {
                (StatusCode::UNAUTHORIZED, self.to_string())
            }

            Self::User(UserError::NotFound)
            | Self::Post(PostError::NotFound)
            | Self::Friendship(FriendshipError::NonexistentUser) => {
                (StatusCode::NOT_FOUND, self.to_string())
            }

            Self::User(UserError::DuplicateEmail | UserError::DuplicateUsername)
            | Self::Friendship(
                FriendshipError::AlreadyFriends | FriendshipError::AlreadyRequested,
            )
            | Self::Post(PostError::DuplicateReply) => (StatusCode::CONFLICT, self.to_string()),

            Self::Post(PostError::DeletedParent) => (StatusCode::GONE, self.to_string()),

            Self::Auth(AuthError::Technical(_))
            | Self::User(UserError::Technical(_))
            | Self::Friendship(FriendshipError::Technical(_))
            | Self::Post(PostError::Technical(_)) => (StatusCode::INTERNAL_SERVER_ERROR, {
                // TODO: use a logger
                eprintln!("{}", self.to_string().red());
                String::from("internal server error")
            }),

            // TODO: This redundant matching on UserError should not be here after the friendship
            // domain is redesigned
            Self::Friendship(FriendshipError::User(err)) => match err {
                UserError::NotFound => (StatusCode::NOT_FOUND, err.to_string()),
                UserError::DuplicateEmail | UserError::DuplicateUsername => {
                    (StatusCode::CONFLICT, err.to_string())
                }
                UserError::Technical(_) => (StatusCode::INTERNAL_SERVER_ERROR, {
                    // TODO: use a logger
                    eprintln!("{}", err.to_string().red());
                    String::from("Internal server error")
                }),
            },
        };

        (status, Json(ErrorResponse { error: message })).into_response()
    }
}
