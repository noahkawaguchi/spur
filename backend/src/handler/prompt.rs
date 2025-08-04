use super::{api_result, validated_json::ValidatedJson};
use crate::{
    domain::content::service::PromptManager,
    dto::{requests::CreatePromptRequest, responses::SinglePromptResponse},
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
        .route("/{prompt_id}", get(get_for_writing))
}

async fn create_new(
    prompt_svc: State<Arc<dyn PromptManager>>,
    Extension(requester_id): Extension<i32>,
    payload: ValidatedJson<CreatePromptRequest>,
) -> api_result!(SinglePromptResponse) {
    let prompt = prompt_svc.create_new(requester_id, &payload.body).await?;
    Ok((StatusCode::CREATED, Json(SinglePromptResponse { prompt })))
}

async fn get_for_writing(
    prompt_svc: State<Arc<dyn PromptManager>>,
    Extension(requester_id): Extension<i32>,
    Path(prompt_id): Path<i32>,
) -> api_result!(SinglePromptResponse) {
    let prompt = prompt_svc.get_for_writing(requester_id, prompt_id).await?;
    Ok((StatusCode::OK, Json(SinglePromptResponse { prompt })))
}
