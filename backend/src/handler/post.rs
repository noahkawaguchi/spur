use super::{api_result, validated_json::ValidatedJson};
use crate::{
    domain::post::PostManager,
    dto::{
        requests::CreatePostRequest,
        responses::{ManyPostsResponse, SinglePostResponse},
    },
    state::AppState,
    utils::vec_into,
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
        .route("/friends", get(all_friend_posts))
        .route("/user/{author_username}", get(specific_user_posts))
        .route("/me", get(own_posts))
}

async fn create_new(
    post_svc: State<Arc<dyn PostManager>>,
    Extension(requester_id): Extension<i32>,
    payload: ValidatedJson<CreatePostRequest>,
) -> api_result!() {
    post_svc
        .create_new(requester_id, payload.parent_id, &payload.body)
        .await?;
    Ok(StatusCode::CREATED)
}

async fn get_by_id(
    post_svc: State<Arc<dyn PostManager>>,
    Path(post_id): Path<i32>,
) -> api_result!(SinglePostResponse) {
    let post = post_svc.get_by_id(post_id).await?.into();
    Ok((StatusCode::OK, Json(SinglePostResponse { post })))
}

async fn all_friend_posts(
    post_svc: State<Arc<dyn PostManager>>,
    Extension(requester_id): Extension<i32>,
) -> api_result!(ManyPostsResponse) {
    let posts = vec_into(post_svc.all_friend_posts(requester_id).await?);
    Ok((StatusCode::OK, Json(ManyPostsResponse { posts })))
}

async fn specific_user_posts(
    post_svc: State<Arc<dyn PostManager>>,
    Path(author_username): Path<String>,
) -> api_result!(ManyPostsResponse) {
    let posts = vec_into(post_svc.user_posts_by_username(&author_username).await?);
    Ok((StatusCode::OK, Json(ManyPostsResponse { posts })))
}

async fn own_posts(
    post_svc: State<Arc<dyn PostManager>>,
    Extension(requester_id): Extension<i32>,
) -> api_result!(ManyPostsResponse) {
    let posts = vec_into(post_svc.user_posts_by_id(requester_id).await?);
    Ok((StatusCode::OK, Json(ManyPostsResponse { posts })))
}
