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
        dto::responses::ErrorResponse,
        test_utils::http_bodies::{deserialize_body, resp_into_body_text},
    };
    use axum::{
        body::Body,
        http::{Method, Request, StatusCode, header::AUTHORIZATION},
    };
    use mockall::predicate::eq;
    use std::sync::Arc;
    use tower::ServiceExt;

    fn create_req(uri: &str, token: Option<&str>) -> Request<Body> {
        let mut req = Request::builder().uri(uri).method(Method::GET);
        if let Some(tok) = token {
            req = req.header(AUTHORIZATION, format!("Bearer {tok}"));
        }
        req.body(Body::empty()).unwrap()
    }

    #[tokio::test]
    async fn does_not_require_auth_for_public_routes() {
        // Auth should not be accessed
        let resp = super::create(AppState::default())
            .oneshot(create_req("/ping", None))
            .await
            .unwrap();

        assert_eq!(StatusCode::OK, resp.status());
        let resp_body = resp_into_body_text(resp).await;
        assert_eq!("pong!", resp_body);
    }

    #[tokio::test]
    async fn allows_access_to_protected_routes_if_authenticated() {
        let token = "bear the bearer";

        let mut mock_auth = MockAuthenticator::new();
        mock_auth
            .expect_validate_token()
            .with(eq(token))
            .once()
            .return_once(|_| Ok(45));

        let state = AppState { auth: Arc::new(mock_auth), ..Default::default() };
        let req = create_req("/auth/check", Some(token));

        let resp = super::create(state).oneshot(req).await.unwrap();
        assert_eq!(StatusCode::OK, resp.status());

        let resp_body = resp_into_body_text(resp).await;
        assert_eq!("Your token is valid", resp_body);
    }

    #[tokio::test]
    async fn disallows_access_to_protected_routes_if_unauthenticated() {
        let token = "bad_token";

        let mut mock_auth = MockAuthenticator::new();
        mock_auth
            .expect_validate_token()
            .with(eq(token))
            .once()
            .return_once(|_| Err(AuthError::TokenValidation));

        let state = AppState { auth: Arc::new(mock_auth), ..Default::default() };
        let req = create_req("/auth/check", Some(token));

        let resp = super::create(state).oneshot(req).await.unwrap();
        assert_eq!(StatusCode::UNAUTHORIZED, resp.status());

        let resp_body = deserialize_body::<ErrorResponse>(resp).await;
        let expected = ErrorResponse {
            error: String::from("Expired or invalid token. Try logging in again."),
        };
        assert_eq!(expected, resp_body);
    }
}
