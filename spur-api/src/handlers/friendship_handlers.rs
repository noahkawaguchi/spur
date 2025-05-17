use crate::{
    error::{ApiError, TechnicalError},
    services::jwt_svc,
};
use axum::{Json, extract::State, http::StatusCode};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use spur_shared::{
    requests::AddFriendRequest,
    responses::{SuccessResponse, UsernamesResponse},
};
use std::sync::Arc;
use validator::Validate;

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait FriendshipManager: Send + Sync {
    /// Attempts to add a friendship between the two users, returning whether or not they are now
    /// friends.
    ///
    /// - If there is a pending request from the recipient to the sender (i.e., an existing request
    /// in the opposite direction), the request is accepted and the two users become friends
    /// (returns true).
    /// - If there is no existing relationship, a new request from the sender to the recipient is
    /// created (returns false).
    ///
    /// # Errors
    ///
    /// Will return `Err` if the two users are already friends, or if there is already a pending
    /// request from the sender to the recipient. (In which case nothing is mutated.)
    async fn add_friend(&self, sender_id: i32, recipient_username: &str) -> Result<bool, ApiError>;

    /// Retrieves the usernames of all confirmed friends of the user with the provided ID.
    async fn get_friends(&self, id: i32) -> Result<Vec<String>, TechnicalError>;

    /// Retrieves the usernames of all users who have pending requests to the user with the
    /// provided ID.
    async fn get_requests(&self, id: i32) -> Result<Vec<String>, TechnicalError>;
}

pub async fn add_friend(
    jwt_secret: State<String>,
    friendship_svc: State<Arc<dyn FriendshipManager>>,
    bearer: TypedHeader<Authorization<Bearer>>,
    payload: Json<AddFriendRequest>,
) -> Result<(StatusCode, Json<SuccessResponse>), ApiError> {
    // Ensure the request body content is valid
    payload.validate()?;

    // User must have a valid token to add a friend
    let sender_id = jwt_svc::validate_jwt(bearer.token(), jwt_secret.as_ref())?;

    // Try to add the friend
    let became_friends = friendship_svc
        .add_friend(sender_id, &payload.recipient_username)
        .await?;

    if became_friends {
        Ok((
            StatusCode::OK,
            Json(SuccessResponse {
                message: format!("You are now friends with {}", payload.recipient_username),
            }),
        ))
    } else {
        Ok((
            StatusCode::CREATED,
            Json(SuccessResponse {
                message: format!("Created a friend request to {}", payload.recipient_username),
            }),
        ))
    }
}

pub async fn get_friends(
    jwt_secret: State<String>,
    friendship_svc: State<Arc<dyn FriendshipManager>>,
    bearer: TypedHeader<Authorization<Bearer>>,
) -> Result<(StatusCode, Json<UsernamesResponse>), ApiError> {
    // User must be authorized
    let id = jwt_svc::validate_jwt(bearer.token(), jwt_secret.as_ref())?;

    // List this user's confirmed friends
    let friends = friendship_svc.get_friends(id).await?;

    Ok((
        StatusCode::OK,
        Json(UsernamesResponse { usernames: friends }),
    ))
}

pub async fn get_requests(
    jwt_secret: State<String>,
    friendship_svc: State<Arc<dyn FriendshipManager>>,
    bearer: TypedHeader<Authorization<Bearer>>,
) -> Result<(StatusCode, Json<UsernamesResponse>), ApiError> {
    // User must be authorized
    let id = jwt_svc::validate_jwt(bearer.token(), jwt_secret.as_ref())?;

    // List pending requests to this user
    let requests = friendship_svc.get_requests(id).await?;

    Ok((
        StatusCode::OK,
        Json(UsernamesResponse { usernames: requests }),
    ))
}
