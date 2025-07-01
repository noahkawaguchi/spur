use super::{api_result, validated_json::ValidatedJson};
use crate::{domain::friendship::service::FriendshipManager, state::AppState};
use axum::{
    Extension, Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use spur_shared::{
    requests::AddFriendRequest,
    responses::{SuccessResponse, UsernamesResponse},
};
use std::sync::Arc;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(add_friend).get(get_friends))
        .route("/requests", get(get_requests))
}

async fn add_friend(
    friendship_svc: State<Arc<dyn FriendshipManager>>,
    Extension(requester_id): Extension<i32>,
    payload: ValidatedJson<AddFriendRequest>,
) -> api_result!(SuccessResponse) {
    // Try to add the friend
    let became_friends = friendship_svc
        .add_friend(requester_id, &payload.recipient_username)
        .await?;

    let (status_code, message) = if became_friends {
        (
            StatusCode::OK,
            format!("You are now friends with {}", payload.recipient_username),
        )
    } else {
        (
            StatusCode::CREATED,
            format!("Created a friend request to {}", payload.recipient_username),
        )
    };

    Ok((status_code, Json(SuccessResponse { message })))
}

async fn get_friends(
    friendship_svc: State<Arc<dyn FriendshipManager>>,
    Extension(requester_id): Extension<i32>,
) -> api_result!(UsernamesResponse) {
    let usernames = friendship_svc.get_friends(requester_id).await?;
    Ok((StatusCode::OK, Json(UsernamesResponse { usernames })))
}

async fn get_requests(
    friendship_svc: State<Arc<dyn FriendshipManager>>,
    Extension(requester_id): Extension<i32>,
) -> api_result!(UsernamesResponse) {
    let usernames = friendship_svc.get_requests(requester_id).await?;
    Ok((StatusCode::OK, Json(UsernamesResponse { usernames })))
}
