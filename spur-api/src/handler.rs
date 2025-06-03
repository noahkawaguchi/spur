pub mod api_error;
pub mod auth;
pub mod content;
pub mod friendship;
pub mod post;
pub mod prompt;

type AuthBearer = axum_extra::TypedHeader<
    axum_extra::headers::Authorization<axum_extra::headers::authorization::Bearer>,
>;

/// Expands to a handler function return type.
///
/// - `api_result!(T)` expands to
///   `Result<(axum::http::StatusCode, axum::Json<T>), crate::handler::api_error::ApiError>`.
///
/// - `api_result!()` expands to
///   `Result<axum::http::StatusCode, crate::handler::api_error::ApiError>`.
macro_rules! api_result {
    ($t:ty) => {
        Result<(axum::http::StatusCode, axum::Json<$t>), crate::handler::api_error::ApiError>
    };
    () => {
        Result<axum::http::StatusCode, crate::handler::api_error::ApiError>
    };
}

pub(crate) use api_result;
