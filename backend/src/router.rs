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
        domain::auth,
        dto::responses::ErrorResponse,
        test_utils::http_bodies::{deserialize_body, resp_into_body_text},
    };
    use axum::{
        body::Body,
        http::{Method, Request, StatusCode, header::AUTHORIZATION},
    };
    use tower::ServiceExt;

    const JWT_SECRET: &str = "top-secret-info";

    fn make_app() -> Router {
        let state = AppState { jwt_secret: JWT_SECRET.to_string(), ..Default::default() };
        super::create(state)
    }

    fn make_req(uri: &str, token: Option<&str>) -> Request<Body> {
        let mut req = Request::builder().uri(uri).method(Method::GET);
        if let Some(tok) = token {
            req = req.header(AUTHORIZATION, format!("Bearer {tok}"));
        }
        req.body(Body::empty()).unwrap()
    }

    #[tokio::test]
    async fn does_not_require_auth_for_public_routes() {
        let resp = make_app().oneshot(make_req("/ping", None)).await.unwrap();

        assert_eq!(StatusCode::OK, resp.status());
        let resp_body = resp_into_body_text(resp).await;
        assert_eq!("pong!", resp_body);
    }

    #[tokio::test]
    async fn allows_access_to_protected_routes_if_authenticated() {
        let token = auth::service::create_test_jwt(7425, JWT_SECRET);
        let req = make_req("/auth/check", Some(&token));

        let resp = make_app().oneshot(req).await.unwrap();
        assert_eq!(StatusCode::OK, resp.status());

        let resp_body = resp_into_body_text(resp).await;
        assert_eq!("Your token is valid", resp_body);
    }

    #[tokio::test]
    async fn disallows_access_to_protected_routes_if_unauthenticated() {
        let req = make_req("/auth/check", Some("invalid"));

        let resp = make_app().oneshot(req).await.unwrap();
        assert_eq!(StatusCode::UNAUTHORIZED, resp.status());

        let resp_body = deserialize_body::<ErrorResponse>(resp).await;
        let expected = ErrorResponse {
            error: String::from("Expired or invalid token. Try logging in again."),
        };
        assert_eq!(expected, resp_body);
    }
}
