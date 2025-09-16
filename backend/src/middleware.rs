use crate::{domain::auth, handler::api_error::ApiError};
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
    let requester_id = auth::service::validate_jwt(bearer.token(), &jwt_secret)?;
    request.extensions_mut().insert(requester_id);
    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        dto::responses::ErrorResponse,
        test_utils::http_bodies::{deserialize_body, resp_into_body_text},
    };
    use axum::{
        Extension, Json, Router,
        body::Body,
        http::{Method, Request, StatusCode, header::AUTHORIZATION},
        middleware,
        routing::get,
    };
    use serde::{Deserialize, Serialize};
    use tower::ServiceExt;

    const JWT_SECRET: &str = "super secret shh";
    const ID_ROUTE: &str = "/my-id";

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct RequesterId {
        requester_id: i32,
    }

    /// Simple handler function that reports the value of the passed extension.
    async fn what_is_my_id(Extension(requester_id): Extension<i32>) -> Json<RequesterId> {
        Json(RequesterId { requester_id })
    }

    /// Makes a GET request to the simple ID reporting endpoint using a router with the JWT
    /// validation middleware applied. If `token` is `Some`, sends the standard `"Authorization":
    /// "Bearer <token>"` header with `token` as the token part, otherwise omits the header.
    async fn make_req(token: Option<&str>) -> Response {
        let mut req = Request::builder().method(Method::GET).uri(ID_ROUTE);
        if let Some(tok) = token {
            req = req.header(AUTHORIZATION, format!("Bearer {tok}"));
        }
        let req_body = req.body(Body::empty()).unwrap();

        Router::new()
            .route(ID_ROUTE, get(what_is_my_id))
            .layer(middleware::from_fn_with_state(
                JWT_SECRET.to_string(),
                validate_jwt,
            ))
            .oneshot(req_body)
            .await
            .unwrap()
    }

    /// Asserts that the response's status is 401 Unauthorized and the body is the expected error
    /// message for an invalid token.
    async fn assert_unauthorized_token(resp: Response) {
        assert_eq!(StatusCode::UNAUTHORIZED, resp.status());
        let resp_body = deserialize_body::<ErrorResponse>(resp).await;
        let expected = ErrorResponse {
            error: String::from("Expired or invalid token. Try logging in again."),
        };
        assert_eq!(expected, resp_body);
    }

    #[tokio::test]
    async fn passes_requester_id_for_valid_token() {
        let requester_id = 654;
        let token = auth::service::create_test_jwt(requester_id, JWT_SECRET);

        let resp = make_req(Some(&token)).await;
        assert_eq!(StatusCode::OK, resp.status());

        let resp_body = deserialize_body::<RequesterId>(resp).await;
        assert_eq!(RequesterId { requester_id }, resp_body);
    }

    #[tokio::test]
    async fn disallows_missing_auth_header() {
        let resp = make_req(None).await;
        assert_eq!(StatusCode::BAD_REQUEST, resp.status());
        let body = resp_into_body_text(resp).await;
        assert_eq!("Header of type `authorization` was missing", body);
    }

    #[tokio::test]
    async fn disallows_empty_auth_header() {
        assert_unauthorized_token(make_req(Some("")).await).await;
    }

    #[tokio::test]
    async fn disallows_bearer_with_no_token() {
        assert_unauthorized_token(make_req(Some("Bearer")).await).await;
    }

    #[tokio::test]
    async fn disallows_invalid_token() {
        assert_unauthorized_token(make_req(Some("Bearer nonsense")).await).await;
    }
}
