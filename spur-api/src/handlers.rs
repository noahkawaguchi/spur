use crate::{
    config::{AppState, DbConfig},
    services::auth_svc::{self, email_username_available, validate_credentials},
};
use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use spur_shared::{
    dto::{ErrorResponse, LoginRequest, LoginResponse, SignupRequest},
    validator::{validate_login_request, validate_signup_request},
};

pub async fn signup(State(db): State<DbConfig>, Json(payload): Json<SignupRequest>) -> Response {
    // Validate the request fields
    if let Err(e) = validate_signup_request(&payload) {
        return (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })).into_response();
    }

    // Check for email and username uniqueness
    if let Err(e) = email_username_available(&db.pool, &payload).await {
        return (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })).into_response();
    }

    // Register the new user
    match auth_svc::register(&db.pool, &payload).await {
        Ok(()) => StatusCode::CREATED.into_response(),
        Err(e) => {
            eprintln!("{}", e); // TODO: use a logger
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: String::from("failed to register") }),
            )
                .into_response()
        }
    }
}

pub async fn login(State(state): State<AppState>, Json(payload): Json<LoginRequest>) -> Response {
    // Validate the request fields
    if let Err(e) = validate_login_request(&payload) {
        return (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })).into_response();
    }

    // Validate the email and password
    let user = match validate_credentials(&state.pool, &payload).await {
        Ok(user) => user,
        Err(e) => {
            return (StatusCode::UNAUTHORIZED, Json(ErrorResponse { error: e })).into_response();
        }
    };

    // Create a JWT
    match auth_svc::create_jwt(user.id, state.jwt_secret.as_ref()) {
        Ok(token) => (StatusCode::OK, Json(LoginResponse { token })).into_response(),
        Err(e) => {
            eprintln!("{}", e); // TODO: use a logger
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: String::from("failed to create JWT") }),
            )
                .into_response()
        }
    }
}
