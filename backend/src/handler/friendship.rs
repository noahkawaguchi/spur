use super::{api_result, validated_json::ValidatedJson};
use crate::{
    app_services::MutateFriendshipByUsername,
    dto::{requests::AddFriendRequest, responses::SuccessResponse},
    read_models::SocialRead,
    state::AppState,
};
use axum::{
    Extension, Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use std::sync::Arc;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(add_friend).get(get_friends))
        .route("/requests", get(get_requests))
}

/// Creates a new friend request or accepts an existing friend request.
async fn add_friend(
    mutate_friendship_by_username: State<Arc<dyn MutateFriendshipByUsername>>,
    Extension(requester_id): Extension<i32>,
    payload: ValidatedJson<AddFriendRequest>,
) -> api_result!(SuccessResponse) {
    // Try to add the friend
    let became_friends = mutate_friendship_by_username
        .add_friend_by_username(requester_id, &payload.recipient_username)
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

/// Retrieves the usernames of the requester's friends.
async fn get_friends(
    social_read: State<Arc<dyn SocialRead>>,
    Extension(requester_id): Extension<i32>,
) -> api_result!(Vec<String>) {
    Ok((
        StatusCode::OK,
        Json(social_read.friend_usernames(requester_id).await?),
    ))
}

/// Retrieves the usernames of users who have pending friend requests to the requester.
async fn get_requests(
    social_read: State<Arc<dyn SocialRead>>,
    Extension(requester_id): Extension<i32>,
) -> api_result!(Vec<String>) {
    Ok((
        StatusCode::OK,
        Json(social_read.pending_requests(requester_id).await?),
    ))
}
