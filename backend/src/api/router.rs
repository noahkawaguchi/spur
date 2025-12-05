use super::{
    handler::{auth, friendship, post},
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
    routing::get,
};
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[allow(clippy::needless_for_each)]
mod docs {
    use crate::api::{
        handler::{auth::docs::AuthDoc, friendship::docs::FriendsDoc},
        utoipa_security::JwtAddon,
    };
    use utoipa::OpenApi;

    #[derive(OpenApi)]
    #[openapi(
        modifiers(&JwtAddon),
        nest(
            (path = "/auth", api = AuthDoc),
            (path = "/friends", api = FriendsDoc),
        ),
    )]
    pub(super) struct ApiDoc;
}

/// Creates the API/web layer and sets it up to accept requests from the provided origin.
pub fn build(state: AppState, frontend_url: &str) -> Result<Router> {
    let cors = CorsLayer::new()
        .allow_origin([frontend_url.parse()?])
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([CONTENT_TYPE, AUTHORIZATION])
        .allow_credentials(true);

    let app = Router::new()
        .route("/ping", get(|| async { "pong!" })) // Simple health check route with no auth
        .nest("/auth", auth::routes().with_state(state.clone())) // The only main public routes
        .merge(protected_routes(state))
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", docs::ApiDoc::openapi()))
        .layer(cors);

    Ok(app)
}

fn protected_routes(state: AppState) -> Router {
    Router::new()
        .route("/auth/check", get(|| async { "Your token is valid" })) // Simple token check route
        .nest("/friends", friendship::routes())
        .nest("/posts", post::routes())
        .route_layer(middleware::from_fn_with_state(state.clone(), validate_jwt))
        .with_state(state)
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
    use axum::{
        body::Body,
        http::{
            HeaderName, Request, Response, StatusCode,
            header::{
                ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_HEADERS,
                ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN,
                ACCESS_CONTROL_REQUEST_HEADERS, ACCESS_CONTROL_REQUEST_METHOD, ORIGIN,
            },
        },
    };
    use mockall::predicate::eq;
    use std::sync::Arc;
    use tower::ServiceExt;

    const TEST_TOKEN: &str = "bear-the-bearer";

    mod auth_requirement {
        use super::*;

        async fn send_req(state: AppState, uri: &str, token: Option<&str>) -> Response<Body> {
            let mut req = Request::builder().uri(uri).method(Method::GET);
            if let Some(tok) = token {
                req = req.header(AUTHORIZATION, format!("Bearer {tok}"));
            }
            super::build(state, "example.com")
                .unwrap()
                .oneshot(req.body(Body::empty()).unwrap())
                .await
                .unwrap()
        }

        #[test]
        fn does_not_require_auth_for_public_route() {
            tokio_test(async {
                // Auth should not be accessed
                let resp = send_req(AppState::default(), "/ping", None).await;
                assert_eq!(StatusCode::OK, resp.status());
                let resp_body = resp_into_body_text(resp).await;
                assert_eq!("pong!", resp_body);
            });
        }

        #[test]
        fn allows_access_to_protected_route_if_authenticated() {
            tokio_test(async {
                let mut mock_auth = MockAuthenticator::new();
                mock_auth
                    .expect_validate_token()
                    .with(eq(TEST_TOKEN))
                    .once()
                    .return_once(|_| Ok(45));

                let state = AppState { auth: Arc::new(mock_auth), ..Default::default() };

                let resp = send_req(state, "/auth/check", Some(TEST_TOKEN)).await;
                assert_eq!(StatusCode::OK, resp.status());

                let resp_body = resp_into_body_text(resp).await;
                assert_eq!("Your token is valid", resp_body);
            });
        }

        #[test]
        fn disallows_access_to_protected_route_if_unauthenticated() {
            tokio_test(async {
                let mut mock_auth = MockAuthenticator::new();
                mock_auth
                    .expect_validate_token()
                    .with(eq(TEST_TOKEN))
                    .once()
                    .return_once(|_| Err(AuthError::TokenValidation));

                let state = AppState { auth: Arc::new(mock_auth), ..Default::default() };

                let resp = send_req(state, "/auth/check", Some(TEST_TOKEN)).await;
                assert_eq!(StatusCode::UNAUTHORIZED, resp.status());

                let resp_body = deserialize_body::<ErrorResponse>(resp).await;
                let expected = ErrorResponse {
                    error: String::from("Expired or invalid token. Try logging in again."),
                };
                assert_eq!(expected, resp_body);
            });
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
        ) -> Response<Body> {
            let mut req = Request::builder().uri(uri).method(&method);
            if let Some(tok) = token {
                req = req.header(AUTHORIZATION, format!("Bearer {tok}"));
            }
            if method == Method::POST {
                req = req.header(CONTENT_TYPE, "application/json");
            }
            super::build(state, "example.com")
                .unwrap()
                .oneshot(req.body(body).unwrap())
                .await
                .unwrap()
        }

        #[test]
        fn passes_state_to_handler_for_public_endpoint() {
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
                    serialize_body(&payload),
                )
                .await;

                assert_eq!(StatusCode::OK, resp.status());
                let resp_body = deserialize_body::<TokenResponse>(resp).await;
                let expected = TokenResponse { token: TEST_TOKEN.to_string() };
                assert_eq!(expected, resp_body);
            });
        }

        #[test]
        fn passes_state_to_handler_for_protected_endpoint() {
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
                .await;

                assert_eq!(StatusCode::OK, resp.status());
                let resp_body = deserialize_body::<Vec<String>>(resp).await;
                assert_eq!(requester_usernames, resp_body);
            });
        }
    }

    mod cors {
        use super::*;

        async fn send_req(
            allowed_origin: &'static str,
            method: Method,
            uri: &'static str,
            req_headers: &[(HeaderName, &str)],
        ) -> Response<Body> {
            let mut req = Request::builder().uri(uri).method(method);
            for &(ref k, v) in req_headers {
                req = req.header(k, v);
            }
            super::build(AppState::default(), allowed_origin)
                .unwrap()
                .oneshot(req.body(Body::empty()).unwrap())
                .await
                .unwrap()
        }

        #[test]
        fn does_not_send_allow_origin_header_if_origin_not_allowed() {
            tokio_test(async {
                let resp = send_req(
                    "https://frontend.example",
                    Method::GET,
                    "/ping",
                    &[(ORIGIN, "https://not-allowed.example")],
                )
                .await;

                assert!(resp.headers().get(ACCESS_CONTROL_ALLOW_ORIGIN).is_none());
            });
        }

        #[test]
        fn sends_correct_cors_headers_for_allowed_origin() {
            tokio_test(async {
                let origin = "https://frontend.example";
                let resp = send_req(origin, Method::GET, "/ping", &[(ORIGIN, origin)]).await;
                let h = resp.headers();
                assert_eq!(origin, h.get(ACCESS_CONTROL_ALLOW_ORIGIN).unwrap());
                assert_eq!("true", h.get(ACCESS_CONTROL_ALLOW_CREDENTIALS).unwrap());
            });
        }

        #[test]
        fn sends_correct_cors_headers_for_preflight_from_allowed_origin() {
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
                .await;

                let h = resp.headers();

                assert_eq!(
                    "https://frontend.example",
                    h.get(ACCESS_CONTROL_ALLOW_ORIGIN).unwrap()
                );
                assert_eq!(
                    "GET,POST,OPTIONS",
                    h.get(ACCESS_CONTROL_ALLOW_METHODS).unwrap()
                );
                assert_eq!(
                    "content-type,authorization",
                    h.get(ACCESS_CONTROL_ALLOW_HEADERS).unwrap()
                );
                assert_eq!("true", h.get(ACCESS_CONTROL_ALLOW_CREDENTIALS).unwrap());
            });
        }
    }
}
