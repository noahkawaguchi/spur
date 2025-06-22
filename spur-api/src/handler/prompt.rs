use super::{AuthBearer, api_result, validated_json::ValidatedJson};
use crate::{domain::content::service::PromptManager, service};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use spur_shared::{requests::CreatePromptRequest, responses::SinglePromptResponse};
use std::sync::Arc;

pub async fn create_new(
    jwt_secret: State<String>,
    prompt_svc: State<Arc<dyn PromptManager>>,
    bearer: AuthBearer,
    payload: ValidatedJson<CreatePromptRequest>,
) -> api_result!(SinglePromptResponse) {
    let requester_id = service::auth::validate_jwt(bearer.token(), &jwt_secret)?;
    let prompt = prompt_svc.create_new(requester_id, &payload.body).await?;

    Ok((StatusCode::CREATED, Json(SinglePromptResponse { prompt })))
}

pub async fn get_for_writing(
    jwt_secret: State<String>,
    prompt_svc: State<Arc<dyn PromptManager>>,
    bearer: AuthBearer,
    Path(prompt_id): Path<i32>,
) -> api_result!(SinglePromptResponse) {
    let requester_id = service::auth::validate_jwt(bearer.token(), &jwt_secret)?;

    let prompt = prompt_svc.get_for_writing(requester_id, prompt_id).await?;

    Ok((StatusCode::OK, Json(SinglePromptResponse { prompt })))
}
