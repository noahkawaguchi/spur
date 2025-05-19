use super::api_error::ApiError;
use crate::{
    config::AppState,
    models::user::{User, UserRegistration},
    services::{domain_error::DomainError, jwt_svc},
    technical_error::TechnicalError,
};
use anyhow::Result;
use axum::{Json, extract::State, http::StatusCode};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use spur_shared::{
    requests::{LoginRequest, SignupRequest},
    responses::LoginResponse,
};
use std::sync::Arc;
use validator::Validate;

#[async_trait::async_trait]
pub trait Authenticator: Send + Sync {
    /// Checks if an account with the given email or username already exists in the database.
    async fn email_username_available(
        &self,
        email: &str,
        username: &str,
    ) -> Result<(), DomainError>;

    /// Hashes the password and creates a new user in the database.
    async fn register(&self, reg: UserRegistration) -> Result<(), DomainError>;

    /// Checks `email` and `password` for a valid match in the database.
    async fn validate_credentials(&self, email: &str, password: &str) -> Result<User, DomainError>;
}

pub async fn signup(
    auth_svc: State<Arc<dyn Authenticator>>,
    Json(payload): Json<SignupRequest>,
) -> Result<StatusCode, ApiError> {
    // Validate the request fields
    payload.validate()?;

    // Check for email and username uniqueness
    auth_svc
        .email_username_available(&payload.email, &payload.username)
        .await?;

    // Register the new user
    auth_svc.register(payload.into()).await?;

    Ok(StatusCode::CREATED)
}

pub async fn login(
    state: State<AppState>,
    payload: Json<LoginRequest>,
) -> Result<(StatusCode, Json<LoginResponse>), ApiError> {
    // Validate the request fields
    payload.validate()?;

    // Validate the email and password
    let user = state
        .auth_svc
        .validate_credentials(&payload.email, &payload.password)
        .await?;

    // Create a JWT
    let token = jwt_svc::create_jwt(user.id, state.jwt_secret.as_ref())
        .map_err(TechnicalError::from)
        .map_err(DomainError::from)?;

    Ok((StatusCode::OK, Json(LoginResponse { token })))
}

pub async fn check(
    jwt_secret: State<String>,
    bearer: TypedHeader<Authorization<Bearer>>,
) -> Result<StatusCode, ApiError> {
    jwt_svc::validate_jwt(bearer.token(), jwt_secret.as_ref())?;
    Ok(StatusCode::NO_CONTENT)
}

// TODO: Now that the handlers are simpler with the help of thiserror, the tests below should
// really be integration tests

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::services::jwt_svc::create_jwt;
//     use crate::{
//         handlers::friendship_handlers::MockFriendshipManager, services::jwt_svc::verify_jwt,
//     };
//     use anyhow::anyhow;
//     use axum_extra::{extract::TypedHeader, headers::Authorization};
//     use chrono::Utc;
//     use mockall::predicate::eq;

//     mod signup {
//         use super::*;

//         #[tokio::test]
//         async fn returns_bad_request_for_invalid_request() {
//             let invalid_request = SignupRequest {
//                 name: String::from("Dan"),
//                 email: String::from("danny@invalid.fail"),
//                 username: String::from("danny_dan"),
//                 password: String::from("insecure_password"),
//             };

//             let mut mock_svc = MockAuthenticator::new();
//             mock_svc.expect_email_username_available().never();
//             mock_svc.expect_register().never();
//             mock_svc.expect_validate_credentials().never();

//             let (status, Json(body)) = signup(State(Arc::new(mock_svc)), Json(invalid_request))
//                 .await
//                 .expect_err("unexpected Ok response");

//             assert_eq!(status, StatusCode::BAD_REQUEST);
//             assert_eq!(
//                 body,
//                 ErrorResponse {
//                     error: String::from(
//                         "password: password must contain at least one uppercase letter"
//                     ),
//                 }
//             );
//         }

//         #[tokio::test]
//         async fn returns_conflict_for_unavailable_credentials() {
//             let already_registered = SignupRequest {
//                 name: String::from("Eunice"),
//                 email: String::from("eunice@registered.xyz"),
//                 username: String::from("eunice_exists"),
//                 password: String::from("passWORD135@$^secURITY"),
//             };

//             let mut mock_svc = MockAuthenticator::new();
//             mock_svc.expect_register().never();
//             mock_svc.expect_validate_credentials().never();
//             mock_svc
//                 .expect_email_username_available()
//                 .with(eq(already_registered.clone()))
//                 .once()
//                 .return_const(Err(String::from(
//                     "an account with the same email already exists",
//                 )));

//             let (status, Json(body)) = signup(State(Arc::new(mock_svc)), Json(already_registered))
//                 .await
//                 .expect_err("unexpected Ok response");

//             assert_eq!(status, StatusCode::CONFLICT);
//             assert_eq!(
//                 body,
//                 ErrorResponse {
//                     error: String::from("an account with the same email already exists")
//                 }
//             );
//         }

//         #[tokio::test]
//         async fn returns_internal_server_error_for_unsuccessful_registration() {
//             let request = SignupRequest {
//                 name: String::from("Frank"),
//                 email: String::from("frank@frank.com"),
//                 username: String::from("frank_man"),
//                 password: String::from("abcABC994&&ad"),
//             };

//             let mut mock_svc = MockAuthenticator::new();
//             mock_svc.expect_validate_credentials().never();
//             mock_svc
//                 .expect_email_username_available()
//                 .with(eq(request.clone()))
//                 .once()
//                 .return_const(Ok(()));
//             mock_svc
//                 .expect_register()
//                 .with(eq(request.clone()))
//                 .once()
//                 .return_once(|_| Err(anyhow!("an error is expected here")));

//             let (status, Json(body)) = signup(State(Arc::new(mock_svc)), Json(request))
//                 .await
//                 .expect_err("unexpected Ok response");

//             assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
//             assert_eq!(
//                 body,
//                 ErrorResponse { error: String::from("failed to register") }
//             );
//         }

//         #[tokio::test]
//         async fn returns_created_for_successful_registration() {
//             let request = SignupRequest {
//                 name: String::from("Frank"),
//                 email: String::from("frank@frank.com"),
//                 username: String::from("frank_man"),
//                 password: String::from("abcABC994&&ad"),
//             };

//             let mut mock_svc = MockAuthenticator::new();
//             mock_svc.expect_validate_credentials().never();
//             mock_svc
//                 .expect_email_username_available()
//                 .with(eq(request.clone()))
//                 .once()
//                 .return_const(Ok(()));
//             mock_svc
//                 .expect_register()
//                 .with(eq(request.clone()))
//                 .once()
//                 .return_once(|_| Ok(()));

//             let status = signup(State(Arc::new(mock_svc)), Json(request))
//                 .await
//                 .expect("error despite successful registration");

//             assert_eq!(status, StatusCode::CREATED);
//         }
//     }

//     mod login {
//         use super::*;

//         #[tokio::test]
//         async fn returns_bad_request_for_invalid_request() {
//             let invalid_request = LoginRequest {
//                 email: String::from("not_an_email"),
//                 password: String::from("my_pass"),
//             };

//             let mut mock_svc = MockAuthenticator::new();
//             mock_svc.expect_email_username_available().never();
//             mock_svc.expect_register().never();
//             mock_svc.expect_validate_credentials().never();

//             let state = AppState {
//                 jwt_secret: String::from("anything here"),
//                 auth_svc: Arc::new(mock_svc),
//                 friendship_svc: Arc::new(MockFriendshipManager::new()),
//             };

//             let (status, Json(body)) = login(State(state), Json(invalid_request))
//                 .await
//                 .expect_err("unexpected Ok response");

//             assert_eq!(status, StatusCode::BAD_REQUEST);
//             assert_eq!(
//                 body,
//                 ErrorResponse { error: String::from("email: not a valid email address") },
//             );
//         }

//         #[tokio::test]
//         async fn returns_unauthorized_for_bad_credentials() {
//             let unregistered = LoginRequest {
//                 email: String::from("unregistered@mail.site"),
//                 password: String::from("my_pass"),
//             };

//             let mut mock_svc = MockAuthenticator::new();
//             mock_svc.expect_email_username_available().never();
//             mock_svc.expect_register().never();
//             mock_svc
//                 .expect_validate_credentials()
//                 .with(eq(unregistered.clone()))
//                 .once()
//                 .return_const(Err(String::from("invalid email")));

//             let state = AppState {
//                 jwt_secret: String::from("anything here"),
//                 auth_svc: Arc::new(mock_svc),
//                 friendship_svc: Arc::new(MockFriendshipManager::new()),
//             };

//             let (status, Json(body)) = login(State(state), Json(unregistered))
//                 .await
//                 .expect_err("unexpected Ok response");

//             assert_eq!(status, StatusCode::UNAUTHORIZED);
//             assert_eq!(body, ErrorResponse { error: String::from("invalid email") });
//         }

//         // NOTE: the case where JWT creation fails is not tested from this module for now

//         #[tokio::test]
//         async fn returns_token_for_successful_login() {
//             let password = String::from("good secure password");
//             let secret = String::from("good secure secret");

//             let greg = User {
//                 id: 1235,
//                 name: String::from("Greg"),
//                 email: String::from("greg@ory.com"),
//                 username: String::from("greg_ory"),
//                 password_hash: bcrypt::hash(&password, bcrypt::DEFAULT_COST)
//                     .expect("failed to hash password"),
//                 created_at: Utc::now(),
//             };

//             let good_request = LoginRequest { email: greg.email.clone(), password };

//             let mut mock_svc = MockAuthenticator::new();
//             mock_svc.expect_email_username_available().never();
//             mock_svc.expect_register().never();
//             mock_svc
//                 .expect_validate_credentials()
//                 .with(eq(good_request.clone()))
//                 .once()
//                 .return_const(Ok(greg.clone()));

//             let state = AppState {
//                 jwt_secret: secret.clone(),
//                 auth_svc: Arc::new(mock_svc),
//                 friendship_svc: Arc::new(MockFriendshipManager::new()),
//             };

//             let (status, Json(body)) = login(State(state), Json(good_request))
//                 .await
//                 .expect("failed to log in");

//             let got_id = verify_jwt(&body.token, secret.as_ref()).expect("failed to verify JWT");

//             assert_eq!(status, StatusCode::OK);
//             assert_eq!(got_id, greg.id);
//         }
//     }

//     mod check {
//         use super::*;

//         #[tokio::test]
//         async fn returns_unauthorized_for_invalid_token() {
//             let secret = String::from("shh");
//             let bearer = TypedHeader(
//                 Authorization::bearer("invalid token").expect("failed to create bearer"),
//             );

//             let (status, Json(body)) = check(State(secret), bearer)
//                 .await
//                 .expect_err("unexpected successful check");

//             assert_eq!(status, StatusCode::UNAUTHORIZED);
//             assert_eq!(
//                 body,
//                 ErrorResponse { error: String::from("expired or invalid token") }
//             );
//         }

//         #[tokio::test]
//         async fn returns_no_content_for_valid_token() {
//             let secret = String::from("good secret");
//             let token = create_jwt(42, secret.as_ref()).expect("failed to create token");
//             let bearer =
//                 TypedHeader(Authorization::bearer(&token).expect("failed to create bearer"));

//             let status = check(State(secret), bearer)
//                 .await
//                 .expect("failed to check token");

//             assert_eq!(status, StatusCode::NO_CONTENT);
//         }
//     }
// }
