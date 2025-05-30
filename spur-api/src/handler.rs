pub mod api_error;
pub mod auth;
pub mod friendship;
pub mod prompt;

type AuthBearer = axum_extra::TypedHeader<
    axum_extra::headers::Authorization<axum_extra::headers::authorization::Bearer>,
>;

/// Expands to a handler function return type.
///
/// - `api_result!(T)` expands to `Result<(StatusCode, Json<T>), ApiError>`.
/// - `api_result!()` expands to `Result<StatusCode, ApiError>`.
macro_rules! api_result {
    ($t:ty) => {
        Result<(StatusCode, axum::Json<$t>), ApiError>
    };
    () => {
        Result<StatusCode, ApiError>
    };
}

pub(crate) use api_result;
