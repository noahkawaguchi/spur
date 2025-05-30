use super::{AuthBearer, api_error::ApiError};
use crate::{domain::prompt::PromptManager, service};
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
    prompt_svc: State<Arc<dyn PromptManager>>,
    bearer: AuthBearer,
    payload: Json<CreatePromptRequest>,
) -> Result<(StatusCode, Json<SinglePromptResponse>), ApiError> {
    payload.validate()?;

    let requester_id = service::auth::validate_jwt(bearer.token(), &jwt_secret)?;
    let prompt = prompt_svc.create_new(requester_id, &payload.body).await?;

    Ok((StatusCode::CREATED, Json(SinglePromptResponse { prompt })))
}

pub async fn get_by_id(
    jwt_secret: State<String>,
    prompt_svc: State<Arc<dyn PromptManager>>,
    bearer: AuthBearer,
    Path(prompt_id): Path<i32>,
) -> Result<(StatusCode, Json<SinglePromptResponse>), ApiError> {
    let requester_id = service::auth::validate_jwt(bearer.token(), &jwt_secret)?;

    let prompt = prompt_svc.get_by_id(requester_id, prompt_id).await?;

    Ok((StatusCode::OK, Json(SinglePromptResponse { prompt })))
}

pub async fn get_by_author(
    jwt_secret: State<String>,
    prompt_svc: State<Arc<dyn PromptManager>>,
    bearer: AuthBearer,
    param: Query<PromptsByAuthorParam>,
) -> Result<(StatusCode, Json<MultiplePromptsResponse>), ApiError> {
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
    prompt_svc: State<Arc<dyn PromptManager>>,
    bearer: AuthBearer,
) -> Result<(StatusCode, Json<MultiplePromptsResponse>), ApiError> {
    let requester_id = service::auth::validate_jwt(bearer.token(), &jwt_secret)?;

    let prompts = prompt_svc.all_friend_prompts(requester_id).await?;

    Ok((StatusCode::OK, Json(MultiplePromptsResponse { prompts })))
}
