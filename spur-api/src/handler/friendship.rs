use super::{api_result, validated_json::ValidatedJson};
use crate::{domain::friendship::service::FriendshipManager, state::AppState};
use axum::{
    Extension, Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::post,
};
use spur_shared::{
    requests::{AddFriendRequest, GetFriendsParam},
    responses::{SuccessResponse, UsernamesResponse},
};
use std::sync::Arc;

pub fn routes() -> Router<AppState> { Router::new().route("/", post(add_friend).get(get_friends)) }

async fn add_friend(
    friendship_svc: State<Arc<dyn FriendshipManager>>,
    Extension(requester_id): Extension<i32>,
    payload: ValidatedJson<AddFriendRequest>,
) -> api_result!(SuccessResponse) {
    // Try to add the friend
    let became_friends = friendship_svc
        .add_friend(requester_id, &payload.recipient_username)
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

async fn get_friends(
    friendship_svc: State<Arc<dyn FriendshipManager>>,
    Extension(requester_id): Extension<i32>,
    param: Query<GetFriendsParam>,
) -> api_result!(UsernamesResponse) {
    let usernames = if param.pending {
        // List pending requests to this user
        friendship_svc.get_requests(requester_id).await?
    } else {
        // List this user's confirmed friends
        friendship_svc.get_friends(requester_id).await?
    };

    Ok((StatusCode::OK, Json(UsernamesResponse { usernames })))
}
