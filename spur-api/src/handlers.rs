use crate::services::auth_svc::{self, email_username_taken};
use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use spur_shared::{
    dto::{ErrorResponse, SignupRequest},
    validator::validate_signup_request,
};
use sqlx::PgPool;

pub async fn signup(State(pool): State<PgPool>, Json(payload): Json<SignupRequest>) -> Response {
    // Validate the request fields
    if let Err(e) = validate_signup_request(&payload) {
        return (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })).into_response();
    }

    // Check for email and username uniqueness
    if let Err(e) = email_username_taken(&pool, &payload).await {
        return (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })).into_response();
    }

    // Register the new user
    match auth_svc::signup(&pool, &payload).await {
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
