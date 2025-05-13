use super::ResponseResult;
use crate::{config::AppState, models::User, services::jwt_svc};
use anyhow::Result;
use axum::{Json, extract::State, http::StatusCode};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use colored::Colorize;
use spur_shared::{
    dto::{ErrorResponse, LoginRequest, LoginResponse, SignupRequest},
    validator::{validate_login_request, validate_signup_request},
};
use std::sync::Arc;

#[async_trait::async_trait]
pub trait AuthService: Send + Sync {
    async fn email_username_available(&self, req: &SignupRequest) -> Result<(), String>;
    async fn register(&self, req: SignupRequest) -> Result<()>;
    async fn validate_credentials(&self, req: &LoginRequest) -> Result<User, String>;
}

pub async fn signup(
    State(auth_svc): State<Arc<dyn AuthService>>,
    Json(payload): Json<SignupRequest>,
) -> ResponseResult<StatusCode> {
    // Validate the request fields
    if let Err(e) = validate_signup_request(&payload) {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })));
    }

    // Check for email and username uniqueness
    if let Err(e) = auth_svc.email_username_available(&payload).await {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })));
    }

    // Register the new user
    match auth_svc.register(payload).await {
        Ok(()) => Ok(StatusCode::CREATED),
        Err(e) => {
            eprintln!("{}", e.to_string().red()); // TODO: use a logger
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: String::from("failed to register") }),
            ))
        }
    }
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> ResponseResult<(StatusCode, Json<LoginResponse>)> {
    // Validate the request fields
    if let Err(e) = validate_login_request(&payload) {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })));
    }

    // Validate the email and password
    let user = match state.auth_svc.validate_credentials(&payload).await {
        Ok(user) => user,
        Err(e) => {
            return Err((StatusCode::UNAUTHORIZED, Json(ErrorResponse { error: e })));
        }
    };

    // Create a JWT
    match jwt_svc::create_jwt(user.id, state.jwt_secret.as_ref()) {
        Ok(token) => Ok((StatusCode::OK, Json(LoginResponse { token }))),
        Err(e) => {
            eprintln!("{}", e.to_string().red()); // TODO: use a logger
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: String::from("failed to create JWT") }),
            ))
        }
    }
}

pub async fn check(
    State(jwt_secret): State<String>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> ResponseResult<StatusCode> {
    match jwt_svc::verify_jwt(bearer.token(), jwt_secret.as_ref()) {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse { error: String::from("expired or invalid token") }),
        )),
    }
}
