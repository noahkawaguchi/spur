use super::{ResponseResult, unauthorized_token};
use crate::services::{friendship_svc::FriendshipOutcome, jwt_svc};
use axum::{Json, extract::State, http::StatusCode};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use colored::Colorize;
use spur_shared::{
    requests::AddFriendRequest,
    responses::{ErrorResponse, SuccessResponse},
};
use std::sync::Arc;

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait FriendshipManager: Send + Sync {
    /// Attempts to add a friendship between the two users. If the they are already friends, or if
    /// there is a pending request from the sender to the recipient, nothing is changed. If there
    /// is a pending request from the recipient to the sender, the request is accepted and the two
    /// users become friends. If there is no existing relationship, a new request from the sender
    /// to the recipient is created.
    async fn add_friend(
        &self,
        sender_id: i32,
        recipient_username: &str,
    ) -> sqlx::Result<FriendshipOutcome>;

    /// Retrieves the usernames of all confirmed friends of the user with the provided ID.
    async fn get_friends(&self, id: i32) -> sqlx::Result<Vec<String>>;

    /// Retrieves the usernames of all users who have pending requests to the user with the
    /// provided ID.
    async fn get_requests(&self, id: i32) -> sqlx::Result<Vec<String>>;
}

pub async fn add_friend(
    jwt_secret: State<String>,
    friendship_svc: State<Arc<dyn FriendshipManager>>,
    bearer: TypedHeader<Authorization<Bearer>>,
    payload: Json<AddFriendRequest>,
) -> ResponseResult<(StatusCode, Json<SuccessResponse>)> {
    // User must have a valid token to add a friend
    let Ok(sender_id) = jwt_svc::verify_jwt(bearer.token(), jwt_secret.as_ref()) else {
        return unauthorized_token();
    };

    // Try to add the friend
    match friendship_svc
        .add_friend(sender_id, &payload.recipient_username)
        .await
    {
        Err(e) => {
            eprintln!("{}", e.to_string().red()); // TODO: use a logger
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: String::from("failed to add friend") }),
            ))
        }

        Ok(FriendshipOutcome::AlreadyFriends) => Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: format!(
                    "You are already friends with {}",
                    payload.recipient_username
                ),
            }),
        )),

        Ok(FriendshipOutcome::AlreadyRequested) => Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: format!(
                    "You already have a pending friend request to {}",
                    payload.recipient_username
                ),
            }),
        )),

        Ok(FriendshipOutcome::BecameFriends) => Ok((
            StatusCode::OK,
            Json(SuccessResponse {
                message: format!("You are now friends with {}", payload.recipient_username),
            }),
        )),

        Ok(FriendshipOutcome::CreatedRequest) => Ok((
            StatusCode::CREATED,
            Json(SuccessResponse {
                message: format!("Created friend request to {}", payload.recipient_username),
            }),
        )),
    }
}
