#![expect(
    clippy::needless_for_each,
    reason = "Inserted when deriving `utoipa::OpenAPI`"
)]

use super::{
    handler::{
        auth::{self, AuthDoc},
        friendship::{self, FriendsDoc},
        post::{self, PostsDoc},
    },
    middleware::validate_jwt,
};
use crate::state::AppState;
use anyhow::Result;
use axum::{
    Router,
    http::{
        Method,
        header::{AUTHORIZATION, CONTENT_TYPE},
    },
    middleware,
    response::Redirect,
    routing::get,
};
use tower_http::cors::CorsLayer;
use utoipa::{
    Modify, OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};
use utoipa_swagger_ui::SwaggerUi;

/// Creates the API/web layer and sets it up to accept requests from the provided origin.
///
/// # Errors
///
/// Returns `Err` if `frontend_url` cannot be parsed.
pub fn build(state: AppState, frontend_url: &str) -> Result<Router> {
    let cors = CorsLayer::new()
        .allow_origin([frontend_url.parse()?])
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([CONTENT_TYPE, AUTHORIZATION])
        .allow_credentials(true);

    let app = Router::new()
        .route("/", get(|| async { Redirect::to("/docs") }))
        .route("/ping", get(pong))
        .nest("/auth", auth::routes().with_state(state.clone()))
        .merge(protected_routes(state))
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(cors);

    Ok(app)
}

fn protected_routes(state: AppState) -> Router {
    Router::new()
        .route("/auth/check", get(token_check))
        .nest("/friends", friendship::routes())
        .nest("/posts", post::routes())
        .route_layer(middleware::from_fn_with_state(state.clone(), validate_jwt))
        .with_state(state)
}

/// Allows simply checking whether the server is running without needing to authenticate.
#[utoipa::path(
    get,
    tag = "health",
    path = "/ping",
    responses((
        status = StatusCode::OK,
        body = &'static str,
        description = "the server is running",
        example = "pong!\n",
    )),
)]
async fn pong() -> &'static str { "pong!\n" }

/// Checks the validity of a JSON Web Token.
#[utoipa::path(
    get,
    tag = "auth",
    path = "/auth/check",
    security(("jwt" = [])),
    responses((
        status = StatusCode::OK,
        body = &'static str,
        description = "token check passed",
        example = "Your token is valid\n",
    )),
)]
async fn token_check() -> &'static str { "Your token is valid\n" }

#[derive(OpenApi)]
#[openapi(
    modifiers(&JwtAddon),
    paths(pong, token_check),
    info(description = API_DESC),
    nest(
        (path = "/auth", api = AuthDoc),
        (path = "/friends", api = FriendsDoc),
        (path = "/posts", api = PostsDoc),
    ),
)]
pub struct ApiDoc;

const API_DESC: &str = "
Spur is a reply-based social platform. More information and the source code are available at \
[github.com/noahkawaguchi/spur](https://github.com/noahkawaguchi/spur).

The docs below detail the various endpoints, and the \"Try it out\" functionality uses the real \
deployed server. Protected endpoints (the ones with a lock symbol) require authentication using a \
JSON Web Token. A token can be acquired via the login or signup endpoints and entered using the \
\"Authorize\" button.

### Common error responses

- All endpoints may return:
  - 400 Bad Request - malformed request
  - 500 Internal Server Error - unexpected technical issue
- All protected endpoints may return:
  - 400 Bad Request - authentication header missing
  - 401 Unauthorized - expired or invalid token
- All POST endpoints may return:
  - 422 Unprocessable Entity - JSON body failed validation logic (e.g. illegal username characters)

Other errors specific to each endpoint are documented below.
";

struct JwtAddon;

impl Modify for JwtAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "jwt",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .description(Some("Enter the token created at signup or login"))
                        .build(),
                ),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        api::dto::{
            dummy_data::dummy_login_request,
            responses::{ErrorResponse, TokenResponse},
        },
        app_services::MockAuthenticator,
        domain::auth::AuthError,
        read_models::MockSocialRead,
        test_utils::{
            http_bodies::{deserialize_body, resp_into_body_text, serialize_body},
            tokio_test,
        },
    };
    use anyhow::Context;
    use axum::{
        body::Body,
        http::{
            HeaderMap, HeaderName, HeaderValue, Method, Request, Response, StatusCode,
            header::{
                ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_HEADERS,
                ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN,
                ACCESS_CONTROL_REQUEST_HEADERS, ACCESS_CONTROL_REQUEST_METHOD, AUTHORIZATION,
                CONTENT_TYPE, ORIGIN,
            },
        },
    };
    use mockall::predicate::eq;
    use std::sync::Arc;
    use tower::ServiceExt;

    const TEST_TOKEN: &str = "bear-the-bearer";

    mod auth_requirement {
        use super::*;

        async fn send_req(
            state: AppState,
            uri: &str,
            token: Option<&str>,
        ) -> Result<Response<Body>> {
            let mut req = Request::builder().uri(uri).method(Method::GET);

            if let Some(tok) = token {
                req = req.header(AUTHORIZATION, format!("Bearer {tok}"));
            }

            super::build(state, "example.com")?
                .oneshot(req.body(Body::empty())?)
                .await
                .map_err(Into::into)
        }

        #[test]
        fn does_not_require_auth_for_public_route() -> Result<()> {
            tokio_test(async {
                // Auth should not be accessed
                let resp = send_req(AppState::default(), "/ping", None).await?;
                assert_eq!(StatusCode::OK, resp.status());
                let resp_body = resp_into_body_text(resp).await?;
                assert_eq!("pong!\n", resp_body);
                Ok(())
            })
        }

        #[test]
        fn allows_access_to_protected_route_if_authenticated() -> Result<()> {
            tokio_test(async {
                let mut mock_auth = MockAuthenticator::new();
                mock_auth
                    .expect_validate_token()
                    .with(eq(TEST_TOKEN))
                    .once()
                    .return_once(|_| Ok(45));

                let state = AppState { auth: Arc::new(mock_auth), ..Default::default() };

                let resp = send_req(state, "/auth/check", Some(TEST_TOKEN)).await?;
                assert_eq!(StatusCode::OK, resp.status());

                let resp_body = resp_into_body_text(resp).await?;
                assert_eq!("Your token is valid\n", resp_body);

                Ok(())
            })
        }

        #[test]
        fn disallows_access_to_protected_route_if_unauthenticated() -> Result<()> {
            tokio_test(async {
                let mut mock_auth = MockAuthenticator::new();
                mock_auth
                    .expect_validate_token()
                    .with(eq(TEST_TOKEN))
                    .once()
                    .return_once(|_| Err(AuthError::TokenValidation));

                let state = AppState { auth: Arc::new(mock_auth), ..Default::default() };

                let resp = send_req(state, "/auth/check", Some(TEST_TOKEN)).await?;

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

    mod state_passing {
        use super::*;

        async fn send_req(
            state: AppState,
            method: Method,
            uri: &str,
            token: Option<&str>,
            body: Body,
        ) -> Result<Response<Body>> {
            let mut req = Request::builder().uri(uri).method(&method);

            if let Some(tok) = token {
                req = req.header(AUTHORIZATION, format!("Bearer {tok}"));
            }
            if method == Method::POST {
                req = req.header(CONTENT_TYPE, "application/json");
            }

            super::build(state, "example.com")?
                .oneshot(req.body(body)?)
                .await
                .map_err(Into::into)
        }

        #[test]
        fn passes_state_to_handler_for_public_endpoint() -> Result<()> {
            tokio_test(async {
                let payload = dummy_login_request();

                let mut mock_auth = MockAuthenticator::new();
                mock_auth
                    .expect_login()
                    .with(eq(payload.email.clone()), eq(payload.password.clone()))
                    .once()
                    .return_once(|_, _| Ok(TEST_TOKEN.to_string()));

                let resp = send_req(
                    AppState { auth: Arc::new(mock_auth), ..Default::default() },
                    Method::POST,
                    "/auth/login",
                    None,
                    serialize_body(&payload)?,
                )
                .await?;

                assert_eq!(StatusCode::OK, resp.status());
                let resp_body = deserialize_body::<TokenResponse>(resp).await?;
                let expected = TokenResponse { token: TEST_TOKEN.to_string() };
                assert_eq!(expected, resp_body);

                Ok(())
            })
        }

        #[test]
        fn passes_state_to_handler_for_protected_endpoint() -> Result<()> {
            tokio_test(async {
                let user_id = 615;
                let requester_usernames = vec![
                    String::from("jun"),
                    String::from("john"),
                    String::from("jessica"),
                    String::from("josue"),
                ];
                let requester_usernames_clone = requester_usernames.clone();

                let mut mock_auth = MockAuthenticator::new();
                mock_auth
                    .expect_validate_token()
                    .with(eq(TEST_TOKEN))
                    .once()
                    .return_once(move |_| Ok(user_id));

                let mut mock_social_read = MockSocialRead::new();
                mock_social_read
                    .expect_pending_requests()
                    .with(eq(user_id))
                    .once()
                    .return_once(move |_| Ok(requester_usernames_clone));

                let resp = send_req(
                    AppState {
                        auth: Arc::new(mock_auth),
                        social_read: Arc::new(mock_social_read),
                        ..Default::default()
                    },
                    Method::GET,
                    "/friends/requests",
                    Some(TEST_TOKEN),
                    Body::empty(),
                )
                .await?;

                assert_eq!(StatusCode::OK, resp.status());
                let resp_body = deserialize_body::<Vec<String>>(resp).await?;
                assert_eq!(requester_usernames, resp_body);

                Ok(())
            })
        }
    }

    mod cors {
        use super::*;

        async fn send_req(
            allowed_origin: &'static str,
            method: Method,
            uri: &'static str,
            req_headers: &[(HeaderName, &str)],
        ) -> Result<Response<Body>> {
            let mut req = Request::builder().uri(uri).method(method);
            for &(ref k, v) in req_headers {
                req = req.header(k, v);
            }
            super::build(AppState::default(), allowed_origin)?
                .oneshot(req.body(Body::empty())?)
                .await
                .map_err(Into::into)
        }

        /// Tries to get the value in `headers` associated with `header_name`.
        fn try_get_header<'a>(
            headers: &'a HeaderMap,
            header_name: &HeaderName,
        ) -> Result<&'a HeaderValue> {
            headers
                .get(header_name)
                .with_context(|| format!("failed to get header {header_name}"))
        }

        #[test]
        fn does_not_send_allow_origin_header_if_origin_not_allowed() -> Result<()> {
            tokio_test(async {
                let resp = send_req(
                    "https://frontend.example",
                    Method::GET,
                    "/ping",
                    &[(ORIGIN, "https://not-allowed.example")],
                )
                .await?;

                assert!(resp.headers().get(ACCESS_CONTROL_ALLOW_ORIGIN).is_none());

                Ok(())
            })
        }

        #[test]
        fn sends_correct_cors_headers_for_allowed_origin() -> Result<()> {
            tokio_test(async {
                let origin = "https://frontend.example";
                let resp = send_req(origin, Method::GET, "/ping", &[(ORIGIN, origin)]).await?;

                let h = resp.headers();
                assert_eq!(origin, try_get_header(h, &ACCESS_CONTROL_ALLOW_ORIGIN)?);
                assert_eq!(
                    "true",
                    try_get_header(h, &ACCESS_CONTROL_ALLOW_CREDENTIALS)?
                );

                Ok(())
            })
        }

        #[test]
        fn sends_correct_cors_headers_for_preflight_from_allowed_origin() -> Result<()> {
            tokio_test(async {
                let origin = "https://frontend.example";

                let resp = send_req(
                    origin,
                    Method::OPTIONS,
                    "/friends",
                    &[
                        (ORIGIN, origin),
                        (ACCESS_CONTROL_REQUEST_METHOD, "POST"),
                        (ACCESS_CONTROL_REQUEST_HEADERS, "content-type,authorization"),
                    ],
                )
                .await?;

                let h = resp.headers();

                assert_eq!(
                    "https://frontend.example",
                    try_get_header(h, &ACCESS_CONTROL_ALLOW_ORIGIN)?
                );
                assert_eq!(
                    "GET,POST,OPTIONS",
                    try_get_header(h, &ACCESS_CONTROL_ALLOW_METHODS)?
                );
                assert_eq!(
                    "content-type,authorization",
                    try_get_header(h, &ACCESS_CONTROL_ALLOW_HEADERS)?
                );
                assert_eq!(
                    "true",
                    try_get_header(h, &ACCESS_CONTROL_ALLOW_CREDENTIALS)?
                );

                Ok(())
            })
        }
    }
}
