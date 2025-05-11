use crate::{
    config::{AppState, DbConfig, JwtConfig},
    services::auth_svc,
};
use axum::{Json, extract::State, http::StatusCode};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use spur_shared::{
    dto::{ErrorResponse, LoginRequest, LoginResponse, SignupRequest},
    validator::{validate_login_request, validate_signup_request},
};

type ResponseResult<T> = Result<T, (StatusCode, Json<ErrorResponse>)>;

pub async fn signup(
    State(db): State<DbConfig>,
    Json(payload): Json<SignupRequest>,
) -> ResponseResult<StatusCode> {
    // Validate the request fields
    if let Err(e) = validate_signup_request(&payload) {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })));
    }

    // Check for email and username uniqueness
    if let Err(e) = auth_svc::email_username_available(&db.pool, &payload).await {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })));
    }

    // Register the new user
    match auth_svc::register(&db.pool, &payload).await {
        Ok(()) => Ok(StatusCode::CREATED),
        Err(e) => {
            eprintln!("{}", e); // TODO: use a logger
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: String::from("failed to register") }),
            ))
        }
    }
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> ResponseResult<(StatusCode, Json<LoginResponse>)> {
    // Validate the request fields
    if let Err(e) = validate_login_request(&payload) {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })));
    }

    // Validate the email and password
    let user = match auth_svc::validate_credentials(&state.pool, &payload).await {
        Ok(user) => user,
        Err(e) => {
            return Err((StatusCode::UNAUTHORIZED, Json(ErrorResponse { error: e })));
        }
    };

    // Create a JWT
    match auth_svc::create_jwt(user.id, state.jwt_secret.as_ref()) {
        Ok(token) => Ok((StatusCode::OK, Json(LoginResponse { token }))),
        Err(e) => {
            eprintln!("{}", e); // TODO: use a logger
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: String::from("failed to create JWT") }),
            ))
        }
    }
}

pub async fn check(
    State(jwt): State<JwtConfig>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> StatusCode {
    match auth_svc::verify_jwt(bearer.token(), jwt.secret.as_ref()) {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(_) => StatusCode::UNAUTHORIZED,
    }
}
