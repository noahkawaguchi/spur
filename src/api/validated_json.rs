use super::error::ApiError;
use axum::{
    Json,
    extract::{FromRequest, Request},
    response::{IntoResponse, Response},
};
use serde::de::DeserializeOwned;
use std::ops::Deref;
use validator::Validate;

/// Custom extractor that validates the request fields using `validator::Validate`.
#[cfg_attr(test, derive(Debug))]
pub struct ValidatedJson<T>(pub T);

impl<T> Deref for ValidatedJson<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // Parse the JSON using Axum's built-in extractor
        let Json(body) = Json::<T>::from_request(req, state)
            .await
            .map_err(IntoResponse::into_response)?;

        // Validate the fields using custom logic
        body.validate()
            .map_err(|e| ApiError::from(e).into_response())?;

        Ok(Self(body))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        api::dto::{responses::ErrorResponse, signup_request::SignupRequest},
        test_utils::{
            http_bodies::{deserialize_body, serialize_body},
            tokio_test,
        },
    };
    use axum::http::{Method, Request, header::CONTENT_TYPE};

    #[test]
    fn allows_valid_json_values() {
        tokio_test(async {
            let payload = SignupRequest {
                name: String::from("Sam Snead"),
                email: String::from("sam@snead.nz"),
                username: String::from("Slam-Sam_54M"),
                password: String::from("#5tr0ngP455W0RD!"),
            };

            let req = Request::builder()
                .uri("/anything")
                .method(Method::POST)
                .header(CONTENT_TYPE, "application/json")
                .body(serialize_body(&payload))
                .unwrap();

            let result = ValidatedJson::<SignupRequest>::from_request(req, &()).await;
            assert!(matches!(result, Ok(ValidatedJson(validated)) if validated == payload));
        });
    }

    #[test]
    fn disallows_invalid_json_values() {
        tokio_test(async {
            let payload = SignupRequest {
                name: String::from("Sam Snead"),
                email: String::from("sam@snead.nz"),
                username: String::from("ユーザーネーム"), // Not allowed!
                password: String::from("#5tr0ngP455W0RD!"),
            };

            let req = Request::builder()
                .uri("/anything")
                .method(Method::POST)
                .header(CONTENT_TYPE, "application/json")
                .body(serialize_body(&payload))
                .unwrap();

            let resp = ValidatedJson::<SignupRequest>::from_request(req, &())
                .await
                .unwrap_err();
            let resp_body = deserialize_body::<ErrorResponse>(resp).await;
            let expected = ErrorResponse {
                error: String::from(
                    "username: username may only contain English letters, \
                    digits, underscores, and hyphens",
                ),
            };

            assert_eq!(expected, resp_body);
        });
    }
}
