use crate::{
    api::dto::responses::ErrorResponse,
    domain::{auth::AuthError, friendship::error::FriendshipError, post::error::PostError},
    read_models::ReadError,
};
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error(transparent)]
    Request(#[from] validator::ValidationErrors),

    #[error(transparent)]
    Auth(#[from] AuthError),

    #[error(transparent)]
    Friendship(#[from] FriendshipError),

    #[error(transparent)]
    Post(#[from] PostError),

    #[error(transparent)]
    Read(#[from] ReadError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            // 401 Unauthorized
            Self::Auth(AuthError::TokenValidation | AuthError::InvalidPassword) => {
                (StatusCode::UNAUTHORIZED, self.to_string())
            }

            // 404 Not Found
            Self::Auth(AuthError::NonexistentAccount)
            | Self::Post(PostError::NotFound)
            | Self::Friendship(FriendshipError::NonexistentUser)
            | Self::Read(ReadError::NotFound) => (StatusCode::NOT_FOUND, self.to_string()),

            // 409 Conflict
            Self::Auth(AuthError::DuplicateEmail | AuthError::DuplicateUsername)
            | Self::Friendship(
                FriendshipError::AlreadyFriends | FriendshipError::AlreadyRequested,
            )
            | Self::Post(PostError::DuplicateReply) => (StatusCode::CONFLICT, self.to_string()),

            // 410 Gone
            Self::Post(PostError::DeletedParent) => (StatusCode::GONE, self.to_string()),

            // 422 Unprocessable Entity
            Self::Request(_)
            | Self::Friendship(FriendshipError::SelfFriendship)
            | Self::Post(PostError::SelfReply | PostError::ArchivedParent) => {
                (StatusCode::UNPROCESSABLE_ENTITY, self.to_string())
            }

            // 500 Internal Server Error
            Self::Auth(AuthError::Internal(_))
            | Self::Friendship(FriendshipError::Internal(_))
            | Self::Post(PostError::Internal(_))
            | Self::Read(ReadError::Technical(_)) => (StatusCode::INTERNAL_SERVER_ERROR, {
                log::error!("{self}");
                String::from("internal server error")
            }),
        };

        (status, Json(ErrorResponse { error: message })).into_response()
    }
}
