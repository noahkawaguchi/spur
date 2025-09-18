use super::{api_result, validated_json::ValidatedJson};
use crate::{
    app_services::Authenticator,
    dto::{requests::LoginRequest, responses::TokenResponse, signup_request::SignupRequest},
    state::AppState,
};
use anyhow::Result;
use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use std::sync::Arc;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/signup", post(signup))
        .route("/login", post(login))
}

async fn signup(
    auth: State<Arc<dyn Authenticator>>,
    ValidatedJson(payload): ValidatedJson<SignupRequest>,
) -> api_result!(TokenResponse) {
    Ok((
        StatusCode::CREATED,
        Json(TokenResponse { token: auth.signup(payload.into()).await? }),
    ))
}

async fn login(
    auth: State<Arc<dyn Authenticator>>,
    payload: ValidatedJson<LoginRequest>,
) -> api_result!(TokenResponse) {
    Ok((
        StatusCode::OK,
        Json(TokenResponse { token: auth.login(&payload.email, &payload.password).await? }),
    ))
}
