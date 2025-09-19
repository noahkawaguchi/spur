use crate::{
    handler::{auth, friendship, post},
    middleware::validate_jwt,
    state::AppState,
};
use axum::{Router, middleware, routing::get};

pub fn create(state: AppState) -> Router {
    Router::new()
        .route("/ping", get(|| async { "pong!" })) // Simple health check route with no auth
        .nest("/auth", auth::routes().with_state(state.clone())) // The only main public routes
        .merge(protected_routes(state))
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
        app_services::MockAuthenticator,
        domain::auth::AuthError,
        dto::responses::{ErrorResponse, TokenResponse},
        read_models::MockSocialRead,
        test_utils::{
            dummy_data,
            http_bodies::{deserialize_body, resp_into_body_text, serialize_body},
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
        super::create(state)
            .oneshot(req.body(body).unwrap())
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn does_not_require_auth_for_public_route() {
        let resp = send_req(
            AppState::default(), // Auth should not be accessed
            Method::GET,
            "/ping",
            None,
            Body::empty(),
        )
        .await;

        assert_eq!(StatusCode::OK, resp.status());
        let resp_body = resp_into_body_text(resp).await;
        assert_eq!("pong!", resp_body);
    }

    #[tokio::test]
    async fn allows_access_to_protected_route_if_authenticated() {
        let mut mock_auth = MockAuthenticator::new();
        mock_auth
            .expect_validate_token()
            .with(eq(TEST_TOKEN))
            .once()
            .return_once(|_| Ok(45));

        let resp = send_req(
            AppState { auth: Arc::new(mock_auth), ..Default::default() },
            Method::GET,
            "/auth/check",
            Some(TEST_TOKEN),
            Body::empty(),
        )
        .await;

        assert_eq!(StatusCode::OK, resp.status());
        let resp_body = resp_into_body_text(resp).await;
        assert_eq!("Your token is valid", resp_body);
    }

    #[tokio::test]
    async fn disallows_access_to_protected_route_if_unauthenticated() {
        let mut mock_auth = MockAuthenticator::new();
        mock_auth
            .expect_validate_token()
            .with(eq(TEST_TOKEN))
            .once()
            .return_once(|_| Err(AuthError::TokenValidation));

        let resp = send_req(
            AppState { auth: Arc::new(mock_auth), ..Default::default() },
            Method::GET,
            "/auth/check",
            Some(TEST_TOKEN),
            Body::empty(),
        )
        .await;

        assert_eq!(StatusCode::UNAUTHORIZED, resp.status());
        let resp_body = deserialize_body::<ErrorResponse>(resp).await;
        let expected = ErrorResponse {
            error: String::from("Expired or invalid token. Try logging in again."),
        };
        assert_eq!(expected, resp_body);
    }

    #[tokio::test]
    async fn passes_state_to_handler_for_public_endpoint() {
        let payload = dummy_data::requests::login();

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
    }

    #[tokio::test]
    async fn passes_state_to_handler_for_protected_endpoint() {
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
    }
}
