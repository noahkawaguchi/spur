use super::{AuthBearer, api_error::ApiError};
use crate::{domain::friendship::service::FriendshipManager, service};
use axum::{Json, extract::State, http::StatusCode};
use spur_shared::{
    requests::AddFriendRequest,
    responses::{SuccessResponse, UsernamesResponse},
};
use std::sync::Arc;
use validator::Validate;

pub async fn add_friend(
    jwt_secret: State<String>,
    friendship_svc: State<Arc<dyn FriendshipManager>>,
    bearer: AuthBearer,
    payload: Json<AddFriendRequest>,
) -> Result<(StatusCode, Json<SuccessResponse>), ApiError> {
    // Ensure the request body content is valid
    payload.validate()?;

    // User must have a valid token to add a friend
    let sender_id = service::auth::validate_jwt(bearer.token(), &jwt_secret)?;

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
    bearer: AuthBearer,
) -> Result<(StatusCode, Json<UsernamesResponse>), ApiError> {
    // User must be authorized
    let id = service::auth::validate_jwt(bearer.token(), &jwt_secret)?;

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
    bearer: AuthBearer,
) -> Result<(StatusCode, Json<UsernamesResponse>), ApiError> {
    // User must be authorized
    let id = service::auth::validate_jwt(bearer.token(), &jwt_secret)?;

    // List pending requests to this user
    let requests = friendship_svc.get_requests(id).await?;

    Ok((
        StatusCode::OK,
        Json(UsernamesResponse { usernames: requests }),
    ))
}
