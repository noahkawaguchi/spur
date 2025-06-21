use anyhow::{Context, Result};
use reqwest::{ClientBuilder, Response};
use serde::Serialize;
use url::Url;

const BUILD_FAILED: &str = "Failed to build request.\nThis can be due to a malformed token.\n\
                            Try using the `login` command to get a new token.";

pub trait RequestClient: Send + Sync {
    async fn get(
        &self,
        endpoint: &str,
        token: &str,
        query_params: Option<impl Serialize>,
    ) -> Result<Response>;

    async fn post(
        &self,
        endpoint: &str,
        body: impl Serialize,
        token: Option<&str>,
    ) -> Result<Response>;
}

#[derive(Clone)]
pub struct ApiRequestClient {
    client: reqwest::Client,
    base_url: Url,
}

impl ApiRequestClient {
    pub fn new(base_url: Url) -> reqwest::Result<Self> {
        let client = ClientBuilder::new().build()?;
        Ok(Self { client, base_url })
    }
}

impl RequestClient for ApiRequestClient {
    async fn get(
        &self,
        endpoint: &str,
        token: &str,
        query_params: Option<impl Serialize>,
    ) -> Result<Response> {
        let url = self.base_url.join(endpoint)?;
        let mut request = self.client.get(url.as_str()).bearer_auth(token);

        if let Some(params) = query_params {
            request = request.query(&params);
        }

        let built_request = request.build().context(BUILD_FAILED)?;

        self.client
            .execute(built_request)
            .await
            .with_context(|| format!("GET request to {url} failed"))
    }

    async fn post(
        &self,
        endpoint: &str,
        body: impl Serialize,
        token: Option<&str>,
    ) -> Result<Response> {
        let url = self.base_url.join(endpoint)?;
        let mut request = self.client.post(url.as_str()).json(&body);

        if let Some(token) = token {
            request = request.bearer_auth(token);
        }

        let built_request = request.build().context(BUILD_FAILED)?;

        self.client
            .execute(built_request)
            .await
            .with_context(|| format!("POST request to {url} failed"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{body_json, header, method, path, query_param},
    };

    mod post {
        use super::*;

        #[tokio::test]
        async fn creates_and_sends_requests() {
            let mock_server = MockServer::start().await;

            let body = json!({"key": "value", "apples": 5});

            Mock::given(method("POST"))
                .and(path("/hello"))
                .and(header("content-type", "application/json"))
                .and(body_json(&body))
                .respond_with(ResponseTemplate::new(200))
                .expect(1)
                .mount(&mock_server)
                .await;

            let base_url = Url::parse(&mock_server.uri()).expect("failed to parse mock server URI");
            let client =
                ApiRequestClient::new(base_url).expect("failed to initialize request client");

            client
                .post("hello", body, None)
                .await
                .expect("failed to make request");
        }

        #[tokio::test]
        async fn optionally_sets_authorization_header() {
            let mock_server = MockServer::start().await;

            let body = json!({"top": "secret", "method": "jwt", "clementines": 1});
            let token = "one or zero clementines";

            Mock::given(method("POST"))
                .and(path("/clementine"))
                .and(header("content-type", "application/json"))
                .and(header("authorization", format!("Bearer {token}")))
                .and(body_json(&body))
                .respond_with(ResponseTemplate::new(200))
                .expect(1)
                .mount(&mock_server)
                .await;

            let base_url = Url::parse(&mock_server.uri()).expect("failed to parse mock server URI");
            let client =
                ApiRequestClient::new(base_url).expect("failed to initialize request client");

            client
                .post("clementine", body, Some(token))
                .await
                .expect("failed to make request");
        }

        #[tokio::test]
        async fn handles_failed_requests() {
            let base = "http://localhost:0";
            let endpoint = "anything";

            let port_zero = Url::parse(base).expect("failed to parse port 0 URL");
            let client =
                ApiRequestClient::new(port_zero).expect("failed to initialize request client");

            let result = client
                .post(endpoint, json!({}), None)
                .await
                .expect_err("unexpected successful request");

            assert_eq!(
                result.to_string(),
                format!("POST request to {base}/{endpoint} failed"),
            );
        }
    }

    mod get {
        use super::*;

        #[tokio::test]
        async fn creates_and_sends_requests() {
            let mock_server = MockServer::start().await;

            let token = "my_secret_token";
            let query_params = &[("bananas", true), ("brown", false)];

            Mock::given(method("GET"))
                .and(path("/banana"))
                .and(header("authorization", format!("Bearer {token}")))
                .and(query_param("bananas", "true"))
                .and(query_param("brown", "false"))
                .respond_with(ResponseTemplate::new(200))
                .expect(1)
                .mount(&mock_server)
                .await;

            let base_url = Url::parse(&mock_server.uri()).expect("failed to parse mock server URI");
            let client =
                ApiRequestClient::new(base_url).expect("failed to initialize request client");

            client
                .get("banana", token, Some(query_params))
                .await
                .expect("failed to make request");
        }

        #[tokio::test]
        async fn handles_failed_requests() {
            let base = "http://localhost:0";
            let endpoint = "nothing";

            let port_zero = Url::parse(base).expect("failed to parse port 0 URL");
            let client =
                ApiRequestClient::new(port_zero).expect("failed to initialize request client");

            let result = client
                .get(endpoint, "token_token", Some(&[("should", "fail")]))
                .await
                .expect_err("unexpected successful request");

            assert_eq!(
                result.to_string(),
                format!("GET request to {base}/{endpoint} failed"),
            );
        }
    }
}
