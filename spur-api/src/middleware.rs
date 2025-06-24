use crate::{handler::api_error::ApiError, service};
use axum::{
    extract::{Request, State},
    middleware,
    response::Response,
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};

/// Middleware that confirms JWT validity and passes the requester's user ID to the handler via a
/// request extension.
pub async fn validate_jwt(
    jwt_secret: State<String>,
    bearer: TypedHeader<Authorization<Bearer>>,
    mut request: Request,
    next: middleware::Next,
) -> Result<Response, ApiError> {
    let requester_id = service::auth::validate_jwt(bearer.token(), &jwt_secret)?;
    request.extensions_mut().insert(requester_id);
    Ok(next.run(request).await)
}
