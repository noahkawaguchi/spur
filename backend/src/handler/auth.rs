use super::{api_result, validated_json::ValidatedJson};
use crate::{
    domain::user::UserManager,
    dto::{
        requests::{LoginRequest, SignupRequest},
        responses::LoginResponse,
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
    user_svc: State<Arc<dyn UserManager>>,
    ValidatedJson(payload): ValidatedJson<SignupRequest>,
) -> api_result!() {
    // Hash the password
    let new_user = service::auth::hash_pw(payload.into())?;

    // Attempt to register the new user
    user_svc.insert_new(&new_user).await?;

    Ok(StatusCode::CREATED)
}

async fn login(
    jwt_secret: State<String>,
    user_svc: State<Arc<dyn UserManager>>,
    payload: ValidatedJson<LoginRequest>,
) -> api_result!(LoginResponse) {
    // Try to get the user
    let user = user_svc.get_by_email(&payload.email).await?;

    // Validate the password and create a JWT
    let token = service::auth::jwt_if_valid_pw(&user, &payload.password, &jwt_secret)?;

    Ok((StatusCode::OK, Json(LoginResponse { token })))
}
