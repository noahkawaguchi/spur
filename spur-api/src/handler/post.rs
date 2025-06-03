use super::{AuthBearer, api_result};
use crate::{domain::content::service::PostManager, service};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use spur_shared::{requests::CreatePostRequest, responses::SinglePostResponse};
use std::sync::Arc;
use validator::Validate;

pub async fn create_new(
    jwt_secret: State<String>,
    post_svc: State<Arc<dyn PostManager>>,
    bearer: AuthBearer,
    payload: Json<CreatePostRequest>,
) -> api_result!(SinglePostResponse) {
    payload.validate()?;

    let requester_id = service::auth::validate_jwt(bearer.token(), &jwt_secret)?;

    let post = post_svc
        .create_new(requester_id, payload.prompt_id, &payload.body)
        .await?;

    Ok((StatusCode::CREATED, Json(SinglePostResponse { post })))
}

pub async fn get_for_reading(
    jwt_secret: State<String>,
    post_svc: State<Arc<dyn PostManager>>,
    bearer: AuthBearer,
    Path(post_id): Path<i32>,
) -> api_result!(SinglePostResponse) {
    let requester_id = service::auth::validate_jwt(bearer.token(), &jwt_secret)?;

    let post = post_svc.get_for_reading(requester_id, post_id).await?;

    Ok((StatusCode::OK, Json(SinglePostResponse { post })))
}
