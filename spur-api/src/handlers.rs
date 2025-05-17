pub mod auth_handlers;
pub mod friendship_handlers;

use axum::{Json, http::StatusCode};
use spur_shared::responses::ErrorResponse;

type ResponseResult<T> = Result<T, (StatusCode, Json<ErrorResponse>)>;

/// Creates a 400 BAD REQUEST response with the provided error message, wrapped in
/// `ResponseResult::Err`.
const fn bad_request<T>(error: String) -> ResponseResult<T> {
    Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error })))
}

/// Creates a 401 UNAUTHORIZED response with the message "expired or invalid token", wrapped in
/// `ResponseResult::Err`.
fn unauthorized_token<T>() -> ResponseResult<T> {
    Err((
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse { error: String::from("expired or invalid token") }),
    ))
}
