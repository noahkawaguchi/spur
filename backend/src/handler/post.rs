use super::{api_result, validated_json::ValidatedJson};
use crate::{
    domain::post::PostSvc,
    dto::{requests::CreatePostRequest, responses::PostResponse},
    map_into::MapInto,
    read_models::{PostWithAuthorRead, SocialRead},
    state::AppState,
};
use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use std::sync::Arc;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_new))
        .route("/{post_id}", get(get_by_id))
        .route("/children/{post_id}", get(get_by_parent_id))
        .route("/friends", get(all_friend_posts))
        .route("/user/{author_username}", get(specific_user_posts))
        .route("/me", get(own_posts))
}

/// Creates a new post.
async fn create_new(
    post_svc: State<Arc<dyn PostSvc>>,
    Extension(requester_id): Extension<i32>,
    payload: ValidatedJson<CreatePostRequest>,
) -> api_result!() {
    post_svc
        .create_new(requester_id, payload.parent_id, &payload.body)
        .await?;

    Ok(StatusCode::CREATED)
}

/// Retrieves a post using its ID.
async fn get_by_id(
    post_with_author_read: State<Arc<dyn PostWithAuthorRead>>,
    Path(post_id): Path<i32>,
) -> api_result!(PostResponse) {
    Ok((
        StatusCode::OK,
        Json(post_with_author_read.by_post_id(post_id).await?.into()),
    ))
}

/// Retrieves the children of the post with the provided ID.
async fn get_by_parent_id(
    post_with_author_read: State<Arc<dyn PostWithAuthorRead>>,
    Path(parent_id): Path<i32>,
) -> api_result!(Vec<PostResponse>) {
    Ok((
        StatusCode::OK,
        Json(post_with_author_read.by_parent(parent_id).await?.map_into()),
    ))
}

/// Retrieves posts written by the requester's friends.
async fn all_friend_posts(
    social_read: State<Arc<dyn SocialRead>>,
    Extension(requester_id): Extension<i32>,
) -> api_result!(Vec<PostResponse>) {
    Ok((
        StatusCode::OK,
        Json(social_read.friend_posts(requester_id).await?.map_into()),
    ))
}

/// Retrieves posts written by the user with the specified username.
async fn specific_user_posts(
    post_with_author_read: State<Arc<dyn PostWithAuthorRead>>,
    Path(author_username): Path<String>,
) -> api_result!(Vec<PostResponse>) {
    Ok((
        StatusCode::OK,
        Json(
            post_with_author_read
                .by_author_username(&author_username)
                .await?
                .map_into(),
        ),
    ))
}

/// Retrieves the requester's own posts.
async fn own_posts(
    post_with_author_read: State<Arc<dyn PostWithAuthorRead>>,
    Extension(requester_id): Extension<i32>,
) -> api_result!(Vec<PostResponse>) {
    Ok((
        StatusCode::OK,
        Json(
            post_with_author_read
                .by_author(requester_id)
                .await?
                .map_into(),
        ),
    ))
}
