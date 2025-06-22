use super::{AuthBearer, api_result, validated_json::ValidatedJson};
use crate::{domain::friendship::service::FriendshipManager, service};
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use spur_shared::{
    requests::{AddFriendRequest, GetFriendsParam},
    responses::{SuccessResponse, UsernamesResponse},
};
use std::sync::Arc;

pub async fn add_friend(
    jwt_secret: State<String>,
    friendship_svc: State<Arc<dyn FriendshipManager>>,
    bearer: AuthBearer,
    payload: ValidatedJson<AddFriendRequest>,
) -> api_result!(SuccessResponse) {
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
    param: Query<GetFriendsParam>,
) -> api_result!(UsernamesResponse) {
    // User must be authorized
    let id = service::auth::validate_jwt(bearer.token(), &jwt_secret)?;

    let usernames = if param.pending {
        // List pending requests to this user
        friendship_svc.get_requests(id).await?
    } else {
        // List this user's confirmed friends
        friendship_svc.get_friends(id).await?
    };

    Ok((StatusCode::OK, Json(UsernamesResponse { usernames })))
}
