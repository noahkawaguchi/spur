use super::{api_result, validated_json::ValidatedJson};
use crate::{
    domain::user::UserManager,
    dto::{
        requests::{LoginRequest, SignupRequest},
        responses::TokenResponse,
    },
    models::user::NewUser,
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
    let password_hash = service::auth::hash_pw(&payload.password)?;

    let new_user = NewUser {
        name: payload.name,
        email: payload.email,
        username: payload.username,
        password_hash,
    };

    let registered_user = user_svc.insert_new(&new_user).await?;

    let token =
        service::auth::create_jwt_if_valid_pw(&registered_user, &payload.password, &jwt_secret)?;

    Ok((StatusCode::CREATED, Json(TokenResponse { token })))
}

async fn login(
    jwt_secret: State<String>,
    user_svc: State<Arc<dyn UserManager>>,
    payload: ValidatedJson<LoginRequest>,
) -> api_result!(TokenResponse) {
    let existing_user = user_svc.get_by_email(&payload.email).await?;

    let token =
        service::auth::create_jwt_if_valid_pw(&existing_user, &payload.password, &jwt_secret)?;

    Ok((StatusCode::OK, Json(TokenResponse { token })))
}
