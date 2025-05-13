pub mod auth_handlers;

type ResponseResult<T> = Result<
    T,
    (
        axum::http::StatusCode,
        axum::Json<spur_shared::dto::ErrorResponse>,
    ),
>;
