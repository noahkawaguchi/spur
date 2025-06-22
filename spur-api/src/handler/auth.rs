use super::{AuthBearer, api_result, validated_json::ValidatedJson};
use crate::{domain::user::UserManager, service};
use anyhow::Result;
use axum::{Json, extract::State, http::StatusCode};
use spur_shared::{
    requests::{LoginRequest, SignupRequest},
    responses::LoginResponse,
};
use std::sync::Arc;

pub async fn signup(
    user_svc: State<Arc<dyn UserManager>>,
    ValidatedJson(payload): ValidatedJson<SignupRequest>,
) -> api_result!() {
    // Hash the password
    let new_user = service::auth::hash_pw(payload.into())?;

    // Attempt to register the new user
    user_svc.insert_new(&new_user).await?;

    Ok(StatusCode::CREATED)
}

pub async fn login(
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

pub async fn check(jwt_secret: State<String>, bearer: AuthBearer) -> api_result!() {
    service::auth::validate_jwt(bearer.token(), &jwt_secret)?;
    Ok(StatusCode::NO_CONTENT)
}
