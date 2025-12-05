use super::{
    handler::{
        auth::{self, docs::AuthDoc},
        friendship::{self, docs::FriendsDoc},
        post::{self, docs::PostsDoc},
    },
    middleware::validate_jwt,
};
use crate::state::AppState;
use axum::{Router, middleware, response::Redirect, routing::get};
use utoipa::{
    Modify, OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};
use utoipa_swagger_ui::SwaggerUi;

/// Sets up the API/web layer.
pub fn build(state: AppState) -> Router {
    Router::new()
        .route("/", get(|| async { Redirect::to("/docs") }))
        .route("/ping", get(pong))
        .nest("/auth", auth::routes().with_state(state.clone()))
        .merge(protected_routes(state))
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", docs::ApiDoc::openapi()))
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
    responses((status = StatusCode::OK, body = &'static str)),
)]
async fn pong() -> &'static str { "pong!\n" }

/// Checks the validity of a JSON Web Token.
#[utoipa::path(
    get,
    tag = "auth",
    path = "/auth/check",
    security(("jwt" = [])),
    responses((status = StatusCode::OK, body = &'static str)),
)]
async fn token_check() -> &'static str { "Your token is valid\n" }

#[allow(clippy::needless_for_each, clippy::wildcard_imports)]
mod docs {
    use super::*;

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
    pub(super) struct ApiDoc;

    const API_DESC: &str = "### Reply-based social platform

#### Repository: [github.com/noahkawaguchi/spur](https://github.com/noahkawaguchi/spur)";

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
            Method, Request, Response, StatusCode,
            header::{AUTHORIZATION, CONTENT_TYPE},
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
            super::build(state)
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
                assert_eq!("pong!\n", resp_body);
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
                assert_eq!("Your token is valid\n", resp_body);
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
            super::build(state)
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
}
