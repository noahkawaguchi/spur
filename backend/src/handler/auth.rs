use super::{api_result, validated_json::ValidatedJson};
use crate::{
    domain::user::UserManager,
    dto::{
        requests::{LoginRequest, SignupRequest},
        responses::TokenResponse,
    },
    service,
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
    jwt_secret: State<String>,
    user_svc: State<Arc<dyn UserManager>>,
    ValidatedJson(payload): ValidatedJson<SignupRequest>,
) -> api_result!(TokenResponse) {
    // Hash the password
    let new_user = service::auth::hash_pw(payload.into())?;

    // Attempt to register the new user
    let id = user_svc.insert_new(&new_user).await?;

    // Create a new JWT
    let token = service::auth::create_jwt(id, &jwt_secret)?;

    Ok((StatusCode::CREATED, Json(TokenResponse { token })))
}

async fn login(
    jwt_secret: State<String>,
    user_svc: State<Arc<dyn UserManager>>,
    payload: ValidatedJson<LoginRequest>,
) -> api_result!(TokenResponse) {
    // Try to get the user
    let user = user_svc.get_by_email(&payload.email).await?;

    // Validate the password and create a JWT
    let token = service::auth::create_jwt_if_valid_pw(&user, &payload.password, &jwt_secret)?;

    Ok((StatusCode::OK, Json(TokenResponse { token })))
}
