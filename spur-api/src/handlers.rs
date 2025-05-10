use crate::models::user::{self, NewUser};
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
    // Validate request fields
    if let Err(e) = validate_signup_request(&payload) {
        return (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })).into_response();
    }

    let hash = "we24gs4"; // TODO

    let new_user = NewUser {
        name: &payload.name,
        email: &payload.email,
        username: &payload.username,
        password_hash: hash,
    };

    match user::insert_new(&pool, &new_user).await {
        Ok(()) => StatusCode::CREATED.into_response(),
        Err(e) => (
            StatusCode::NOT_IMPLEMENTED,
            Json(ErrorResponse { error: format!("not implemented yet! {e}") }),
        )
            .into_response(),
    }
}
