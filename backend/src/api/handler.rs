pub mod auth;
pub mod friendship;
pub mod post;

/// Expands to a handler function return type.
///
/// `api_result!(T)` expands to:
/// `Result<(axum::http::StatusCode, axum::Json<T>), crate::api::error::ApiError>`
///
/// `api_result!()` expands to:
/// `Result<axum::http::StatusCode, crate::api::error::ApiError>`
macro_rules! api_result {
    ($t:ty) => {
        Result<(axum::http::StatusCode, axum::Json<$t>), crate::api::error::ApiError>
    };
    () => {
        Result<axum::http::StatusCode, crate::api::error::ApiError>
    };
}

use api_result;
