pub mod auth_handlers;
pub mod friendship_handlers;

use axum::{Json, http::StatusCode};
use spur_shared::responses::ErrorResponse;

type ResponseResult<T> = Result<T, (StatusCode, Json<ErrorResponse>)>;

/// Creates a 400 BAD REQUEST response with the provided error message, wrapped in Err.
const fn bad_request<T>(error: String) -> ResponseResult<T> {
    Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error })))
}
