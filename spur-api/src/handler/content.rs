use super::api_result;
use crate::{config::AppState, domain::content::service::ContentManager};
use axum::{
    Extension, Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::get,
};
use spur_shared::{requests::UserContentParam, responses::PromptsAndPostsResponse};
use std::sync::Arc;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(user_content))
        .route("/friends", get(friends_content))
}

async fn user_content(
    content_svc: State<Arc<dyn ContentManager>>,
    Extension(requester_id): Extension<i32>,
    param: Query<UserContentParam>,
) -> api_result!(PromptsAndPostsResponse) {
    let (prompts, posts) = if let Some(ref friend_username) = param.author_username {
        content_svc
            .specific_friend_content(requester_id, friend_username)
            .await?
    } else {
        content_svc.own_content(requester_id).await?
    };

    Ok((
        StatusCode::OK,
        Json(PromptsAndPostsResponse { prompts, posts }),
    ))
}

async fn friends_content(
    prompt_svc: State<Arc<dyn ContentManager>>,
    Extension(requester_id): Extension<i32>,
) -> api_result!(PromptsAndPostsResponse) {
    let (prompts, posts) = prompt_svc.all_friend_content(requester_id).await?;

    Ok((
        StatusCode::OK,
        Json(PromptsAndPostsResponse { prompts, posts }),
    ))
}
