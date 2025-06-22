use super::api_error::ApiError;
use axum::{
    Json,
    extract::{FromRequest, Request},
    response::{IntoResponse, Response},
};
use serde::de::DeserializeOwned;
use std::ops::Deref;
use validator::Validate;

/// Custom extractor that validates the request fields using `.validate()`.
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
