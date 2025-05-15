use anyhow::{Context, Result};
use reqwest::{ClientBuilder, Response};
use serde::Serialize;
use url::Url;

pub trait ApiRequest: Send + Sync {
    async fn post<S: Serialize>(&self, endpoint: &str, body: S) -> Result<Response>;
    async fn get(&self, endpoint: &str, token: &str) -> Result<Response>;
}

pub struct BackendRequest {
    client: reqwest::Client,
    base_url: Url,
}

impl BackendRequest {
    /// Creates a request client.
    pub fn new(base_url: Url) -> reqwest::Result<Self> {
        let client = ClientBuilder::new().build()?;
        Ok(Self { client, base_url })
    }
}

impl ApiRequest for BackendRequest {
    async fn post<S: Serialize>(&self, endpoint: &str, body: S) -> Result<Response> {
        let url = self.base_url.join(endpoint)?;

        self.client
            .post(url.as_str())
            .json(&body)
            .send()
            .await
            .with_context(|| format!("POST request to {url} failed"))
    }

    async fn get(&self, endpoint: &str, token: &str) -> Result<Response> {
        let url = self.base_url.join(endpoint)?;

        self.client
            .get(url.as_str())
            .bearer_auth(token)
            .send()
            .await
            .with_context(|| format!("GET request to {url} failed"))
    }
}
