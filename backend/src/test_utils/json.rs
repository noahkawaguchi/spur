use axum::{
    body::{Body, to_bytes},
    response::Response,
};
use serde::{Deserialize, Serialize};

pub fn serialize_body(body: &impl Serialize) -> Body {
    Body::from(serde_json::to_vec(body).unwrap())
}

pub async fn deserialize_body<T: for<'a> Deserialize<'a>>(resp: Response) -> T {
    serde_json::from_slice(&to_bytes(resp.into_body(), usize::MAX).await.unwrap()).unwrap()
}
