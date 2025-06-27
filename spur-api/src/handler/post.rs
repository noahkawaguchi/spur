use super::{api_result, validated_json::ValidatedJson};
use crate::{domain::content::service::PostManager, state::AppState};
use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use spur_shared::{requests::CreatePostRequest, responses::SinglePostResponse};
use std::sync::Arc;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_new))
        .route("/{post_id}", get(get_for_reading))
}

async fn create_new(
    post_svc: State<Arc<dyn PostManager>>,
    Extension(requester_id): Extension<i32>,
    payload: ValidatedJson<CreatePostRequest>,
) -> api_result!(SinglePostResponse) {
    let post = post_svc
        .create_new(requester_id, payload.prompt_id, &payload.body)
        .await?;
    Ok((StatusCode::CREATED, Json(SinglePostResponse { post })))
}

async fn get_for_reading(
    post_svc: State<Arc<dyn PostManager>>,
    Extension(requester_id): Extension<i32>,
    Path(post_id): Path<i32>,
) -> api_result!(SinglePostResponse) {
    let post = post_svc.get_for_reading(requester_id, post_id).await?;
    Ok((StatusCode::OK, Json(SinglePostResponse { post })))
}
