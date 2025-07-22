use super::api_result;
use crate::{
    domain::content::service::ContentManager, dto::responses::PromptsAndPostsResponse,
    state::AppState,
};
use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use std::sync::Arc;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(all_friend_content))
        .route("/friend/{username}", get(specific_friend_content))
        .route("/me", get(own_content))
}

async fn all_friend_content(
    content_svc: State<Arc<dyn ContentManager>>,
    Extension(requester_id): Extension<i32>,
) -> api_result!(PromptsAndPostsResponse) {
    let (prompts, posts) = content_svc.all_friend_content(requester_id).await?;

    Ok((
        StatusCode::OK,
        Json(PromptsAndPostsResponse { prompts, posts }),
    ))
}

async fn specific_friend_content(
    content_svc: State<Arc<dyn ContentManager>>,
    Extension(requester_id): Extension<i32>,
    Path(friend_username): Path<String>,
) -> api_result!(PromptsAndPostsResponse) {
    let (prompts, posts) = content_svc
        .specific_friend_content(requester_id, &friend_username)
        .await?;

    Ok((
        StatusCode::OK,
        Json(PromptsAndPostsResponse { prompts, posts }),
    ))
}

async fn own_content(
    content_svc: State<Arc<dyn ContentManager>>,
    Extension(requester_id): Extension<i32>,
) -> api_result!(PromptsAndPostsResponse) {
    let (prompts, posts) = content_svc.own_content(requester_id).await?;

    Ok((
        StatusCode::OK,
        Json(PromptsAndPostsResponse { prompts, posts }),
    ))
}
