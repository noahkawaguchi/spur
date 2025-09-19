use super::{api_result, validated_json::ValidatedJson};
use crate::{
    app_services::Authenticator,
    dto::{requests::LoginRequest, responses::TokenResponse, signup_request::SignupRequest},
    state::AppState,
};
use anyhow::Result;
use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use std::sync::Arc;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/signup", post(signup))
        .route("/login", post(login))
}

async fn signup(
    auth: State<Arc<dyn Authenticator>>,
    ValidatedJson(payload): ValidatedJson<SignupRequest>,
) -> api_result!(TokenResponse) {
    Ok((
        StatusCode::CREATED,
        Json(TokenResponse { token: auth.signup(payload.into()).await? }),
    ))
}

async fn login(
    auth: State<Arc<dyn Authenticator>>,
    payload: ValidatedJson<LoginRequest>,
) -> api_result!(TokenResponse) {
    Ok((
        StatusCode::OK,
        Json(TokenResponse { token: auth.login(&payload.email, &payload.password).await? }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        app_services::MockAuthenticator,
        domain::auth::AuthError,
        dto::{responses::ErrorResponse, signup_request::SignupRequest},
        models::user::UserRegistration,
        test_utils::http_bodies::{deserialize_body, serialize_body},
    };
    use axum::{
        body::Body,
        http::{Method, Request, Response, header::CONTENT_TYPE},
    };
    use mockall::predicate::eq;
    use serde::Serialize;
    use tower::ServiceExt;

    async fn send_req(
        mock_auth: impl Authenticator + 'static,
        endpoint: &'static str,
        payload: &(impl Serialize + Sync),
    ) -> Response<Body> {
        let state = AppState { auth: Arc::new(mock_auth), ..Default::default() };
        let app = super::routes().with_state(state);
        let req = Request::builder()
            .method(Method::POST)
            .uri(endpoint)
            .header(CONTENT_TYPE, "application/json")
            .body(serialize_body(payload))
            .unwrap();
        app.oneshot(req).await.unwrap()
    }

    mod signup {
        use super::*;

        fn create_signup_payload() -> SignupRequest {
            SignupRequest {
                name: String::from("Christina Ani-Tsi RHC"),
                email: String::from("name@backwards.moc"),
                username: String::from("chris_and_tina"),
                password: String::from("2shh!5hh#H"),
            }
        }

        #[tokio::test]
        async fn returns_token_for_successful_signup() {
            let payload = create_signup_payload();
            let token = "t-0-k-3-n";

            let mut mock_auth = MockAuthenticator::new();
            mock_auth
                .expect_signup()
                .with(eq(UserRegistration::from(payload.clone())))
                .once()
                .return_once(|_| Ok(token.to_string()));

            let resp = send_req(mock_auth, "/signup", &payload).await;
            assert_eq!(resp.status(), StatusCode::CREATED);

            let resp_body = deserialize_body::<TokenResponse>(resp).await;
            let expected = TokenResponse { token: token.to_string() };
            assert_eq!(expected, resp_body);
        }

        #[tokio::test]
        async fn translates_errors() {
            let payload = create_signup_payload();

            let mut mock_auth = MockAuthenticator::new();
            mock_auth
                .expect_signup()
                .with(eq(UserRegistration::from(payload.clone())))
                .once()
                .return_once(|_| Err(AuthError::DuplicateUsername));

            let resp = send_req(mock_auth, "/signup", &payload).await;
            assert_eq!(resp.status(), StatusCode::CONFLICT);

            let resp_body = deserialize_body::<ErrorResponse>(resp).await;
            let expected = ErrorResponse { error: String::from("Username taken") };
            assert_eq!(expected, resp_body);
        }
    }

    mod login {
        use super::*;

        fn create_login_payload() -> LoginRequest {
            LoginRequest {
                email: String::from("name@backwards.moc"),
                password: String::from("2shh!5hh#H"),
            }
        }

        #[tokio::test]
        async fn returns_token_for_successful_login() {
            let payload = create_login_payload();
            let token = "t-0-k-3-n";

            let mut mock_auth = MockAuthenticator::new();
            mock_auth
                .expect_login()
                .with(eq(payload.email.clone()), eq(payload.password.clone()))
                .once()
                .return_once(|_, _| Ok(token.to_string()));

            let resp = send_req(mock_auth, "/login", &payload).await;
            assert_eq!(resp.status(), StatusCode::OK);

            let resp_body = deserialize_body::<TokenResponse>(resp).await;
            let expected = TokenResponse { token: token.to_string() };
            assert_eq!(expected, resp_body);
        }

        #[tokio::test]
        async fn translates_errors() {
            let payload = create_login_payload();

            let mut mock_auth = MockAuthenticator::new();
            mock_auth
                .expect_login()
                .with(eq(payload.email.clone()), eq(payload.password.clone()))
                .once()
                .return_once(|_, _| Err(AuthError::InvalidPassword));

            let resp = send_req(mock_auth, "/login", &payload).await;
            assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

            let resp_body = deserialize_body::<ErrorResponse>(resp).await;
            let expected = ErrorResponse { error: String::from("Invalid password") };
            assert_eq!(expected, resp_body);
        }
    }
}
