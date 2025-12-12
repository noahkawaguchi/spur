use anyhow::Result;
use axum::{
    body::{Body, to_bytes},
    response::Response,
};
use serde::{Deserialize, Serialize};

pub fn serialize_body(body: &impl Serialize) -> Result<Body> {
    Ok(Body::from(serde_json::to_vec(body)?))
}

pub async fn deserialize_body<T: for<'a> Deserialize<'a>>(resp: Response) -> Result<T> {
    serde_json::from_slice(&to_bytes(resp.into_body(), usize::MAX).await?).map_err(Into::into)
}

pub async fn resp_into_body_text(resp: Response) -> Result<String> {
    String::from_utf8(to_bytes(resp.into_body(), usize::MAX).await?.to_vec()).map_err(Into::into)
}
