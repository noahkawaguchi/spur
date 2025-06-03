use super::{AuthBearer, api_result};
use crate::{domain::content::service::ContentManager, service};
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use spur_shared::{requests::UserContentParam, responses::PromptsAndPostsResponse};
use std::sync::Arc;

pub async fn user_content(
    jwt_secret: State<String>,
    content_svc: State<Arc<dyn ContentManager>>,
    bearer: AuthBearer,
    param: Query<UserContentParam>,
) -> api_result!(PromptsAndPostsResponse) {
    let requester_id = service::auth::validate_jwt(bearer.token(), &jwt_secret)?;

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

pub async fn friends_content(
    jwt_secret: State<String>,
    prompt_svc: State<Arc<dyn ContentManager>>,
    bearer: AuthBearer,
) -> api_result!(PromptsAndPostsResponse) {
    let requester_id = service::auth::validate_jwt(bearer.token(), &jwt_secret)?;

    let (prompts, posts) = prompt_svc.all_friend_content(requester_id).await?;

    Ok((
        StatusCode::OK,
        Json(PromptsAndPostsResponse { prompts, posts }),
    ))
}
