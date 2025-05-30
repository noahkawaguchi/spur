use super::{AuthBearer, api_error::ApiError, api_result};
use crate::{domain::prompt::ContentManager, service};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use spur_shared::{
    requests::{CreatePromptRequest, PromptsByAuthorParam},
    responses::{MultiplePromptsResponse, SinglePromptResponse},
};
use std::sync::Arc;
use validator::Validate;

pub async fn new_prompt(
    jwt_secret: State<String>,
    prompt_svc: State<Arc<dyn ContentManager>>,
    bearer: AuthBearer,
    payload: Json<CreatePromptRequest>,
) -> api_result!(SinglePromptResponse) {
    payload.validate()?;

    let requester_id = service::auth::validate_jwt(bearer.token(), &jwt_secret)?;
    let prompt = prompt_svc.new_prompt(requester_id, &payload.body).await?;

    Ok((StatusCode::CREATED, Json(SinglePromptResponse { prompt })))
}

pub async fn get_for_writing(
    jwt_secret: State<String>,
    prompt_svc: State<Arc<dyn ContentManager>>,
    bearer: AuthBearer,
    Path(prompt_id): Path<i32>,
) -> api_result!(SinglePromptResponse) {
    let requester_id = service::auth::validate_jwt(bearer.token(), &jwt_secret)?;

    let prompt = prompt_svc
        .get_prompt_for_writing(requester_id, prompt_id)
        .await?;

    Ok((StatusCode::OK, Json(SinglePromptResponse { prompt })))
}

pub async fn get_by_author(
    jwt_secret: State<String>,
    prompt_svc: State<Arc<dyn ContentManager>>,
    bearer: AuthBearer,
    param: Query<PromptsByAuthorParam>,
) -> api_result!(MultiplePromptsResponse) {
    let requester_id = service::auth::validate_jwt(bearer.token(), &jwt_secret)?;

    let prompts = if let Some(ref friend_username) = param.author_username {
        prompt_svc
            .specific_friend_prompts(requester_id, friend_username)
            .await?
    } else {
        prompt_svc.own_prompts(requester_id).await?
    };

    Ok((StatusCode::OK, Json(MultiplePromptsResponse { prompts })))
}

pub async fn all_friend_prompts(
    jwt_secret: State<String>,
    prompt_svc: State<Arc<dyn ContentManager>>,
    bearer: AuthBearer,
) -> api_result!(MultiplePromptsResponse) {
    let requester_id = service::auth::validate_jwt(bearer.token(), &jwt_secret)?;

    let prompts = prompt_svc.all_friend_prompts(requester_id).await?;

    Ok((StatusCode::OK, Json(MultiplePromptsResponse { prompts })))
}
