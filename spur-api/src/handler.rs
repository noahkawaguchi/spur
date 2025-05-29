pub mod api_error;
pub mod auth;
pub mod friendship;
pub mod prompt;

type AuthBearer = axum_extra::TypedHeader<
    axum_extra::headers::Authorization<axum_extra::headers::authorization::Bearer>,
>;
