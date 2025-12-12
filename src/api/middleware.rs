use crate::{api::error::ApiError, app_services::Authenticator};
use axum::{
    extract::{Request, State},
    middleware,
    response::Response,
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use std::sync::Arc;

/// Middleware that confirms JWT validity and passes the requester's user ID to the handler via a
/// request extension.
pub async fn validate_jwt(
    auth: State<Arc<dyn Authenticator>>,
    bearer: TypedHeader<Authorization<Bearer>>,
    mut request: Request,
    next: middleware::Next,
) -> Result<Response, ApiError> {
    request
        .extensions_mut()
        .insert(auth.validate_token(bearer.token())?);

    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        api::dto::responses::ErrorResponse,
        app_services::MockAuthenticator,
        domain::auth::AuthError,
        state::AppState,
        test_utils::{
            http_bodies::{deserialize_body, resp_into_body_text},
            tokio_test,
        },
    };
    use anyhow::Result;
    use axum::{
        Extension, Json, Router,
        body::Body,
        http::{Method, Request, StatusCode, header::AUTHORIZATION},
        middleware,
        routing::get,
    };
    use mockall::predicate::eq;
    use serde::{Deserialize, Serialize};
    use tower::ServiceExt;

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
    /// validation middleware applied. If `header_val` is `Some`, sends an "Authorization" header
    /// with `header_val` as the value, otherwise omits the header.
    async fn send_req(
        header_val: Option<&str>,
        mock_auth: impl Authenticator + 'static,
    ) -> Result<Response> {
        let mut req = Request::builder().method(Method::GET).uri(ID_ROUTE);
        if let Some(bearer_tok) = header_val {
            req = req.header(AUTHORIZATION, bearer_tok);
        }
        let req_body = req.body(Body::empty())?;

        Router::new()
            .route(ID_ROUTE, get(what_is_my_id))
            .layer(middleware::from_fn_with_state(
                AppState { auth: Arc::new(mock_auth), ..Default::default() },
                validate_jwt,
            ))
            .oneshot(req_body)
            .await
            .map_err(Into::into)
    }

    #[test]
    fn passes_requester_id_for_valid_token() -> Result<()> {
        tokio_test(async {
            let requester_id = 654;
            let token = "This token is valid!!1!";

            let mut mock_auth = MockAuthenticator::new();
            mock_auth
                .expect_validate_token()
                .with(eq(token))
                .once()
                .return_once(move |_| Ok(requester_id));

            let resp = send_req(Some(&format!("Bearer {token}")), mock_auth).await?;
            assert_eq!(StatusCode::OK, resp.status());

            let resp_body = deserialize_body::<RequesterId>(resp).await?;
            assert_eq!(RequesterId { requester_id }, resp_body);

            Ok(())
        })
    }

    #[test]
    fn disallows_missing_auth_header() -> Result<()> {
        tokio_test(async {
            let resp = send_req(None, MockAuthenticator::new()).await?;
            assert_eq!(StatusCode::BAD_REQUEST, resp.status());
            let body = resp_into_body_text(resp).await?;
            assert_eq!("Header of type `authorization` was missing", body);
            Ok(())
        })
    }

    #[test]
    fn disallows_empty_auth_header() -> Result<()> {
        tokio_test(async {
            let resp = send_req(Some(""), MockAuthenticator::new()).await?;
            assert_eq!(StatusCode::BAD_REQUEST, resp.status());
            let body = resp_into_body_text(resp).await?;
            assert_eq!("invalid HTTP header (authorization)", body);
            Ok(())
        })
    }

    #[test]
    fn disallows_bearer_with_no_token() -> Result<()> {
        tokio_test(async {
            let resp = send_req(Some("Bearer"), MockAuthenticator::new()).await?;
            assert_eq!(StatusCode::BAD_REQUEST, resp.status());
            let body = resp_into_body_text(resp).await?;
            assert_eq!("invalid HTTP header (authorization)", body);
            Ok(())
        })
    }

    #[test]
    fn disallows_invalid_token() -> Result<()> {
        tokio_test(async {
            let token = "nonsense";

            let mut mock_auth = MockAuthenticator::new();
            mock_auth
                .expect_validate_token()
                .with(eq(token))
                .once()
                .return_once(|_| Err(AuthError::TokenValidation));

            let resp = send_req(Some(&format!("Bearer {token}")), mock_auth).await?;
            assert_eq!(StatusCode::UNAUTHORIZED, resp.status());

            let resp_body = deserialize_body::<ErrorResponse>(resp).await?;
            let expected = ErrorResponse {
                error: String::from("Expired or invalid token. Try logging in again."),
            };
            assert_eq!(expected, resp_body);

            Ok(())
        })
    }
}
