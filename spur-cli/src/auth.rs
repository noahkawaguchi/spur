use crate::error_response;
use anyhow::{Context, Result, anyhow};
use inquire::error::InquireResult;
use reqwest::{ClientBuilder, StatusCode};
use spur_shared::dto::{LoginRequest, LoginResponse, SignupRequest};
use url::Url;

pub trait AuthPrompt: Send + Sync {
    /// Prompts the user for name, email, username, and password.
    fn signup(&self) -> InquireResult<SignupRequest>;
    /// Prompts the user for email and password.
    fn login(&self) -> InquireResult<LoginRequest>;
}

pub trait TokenStore: Send + Sync {
    /// Saves the token to a text file.
    fn save(&self, token: &str) -> Result<()>;
    /// Reads the saved token if it exists.
    fn load(&self) -> Result<String>;
}

pub struct AuthCmd<P, S>
where
    P: AuthPrompt,
    S: TokenStore,
{
    pub prompt: P,
    pub store: S,
}

impl<P, S> AuthCmd<P, S>
where
    P: AuthPrompt,
    S: TokenStore,
{
    pub async fn signup(&self, backend_url: &Url) -> Result<String> {
        let body = self.prompt.signup()?;

        let response = ClientBuilder::new()
            .build()?
            .post(backend_url.join("signup")?)
            .json(&body)
            .send()
            .await
            .context("request failed")?;

        if response.status() == StatusCode::CREATED {
            Ok(String::from("Successfully registered"))
        } else {
            Err(anyhow!(error_response::handle(response).await))
        }
    }

    pub async fn login(&self, backend_url: &Url) -> Result<String> {
        let body = self.prompt.login()?;

        let response = ClientBuilder::new()
            .build()?
            .post(backend_url.join("login")?)
            .json(&body)
            .send()
            .await
            .context("request failed")?;

        if response.status() == StatusCode::OK {
            match self
                .store
                .save(&response.json::<LoginResponse>().await?.token)
            {
                Ok(()) => Ok(String::from("Successfully logged in and saved token")),
                Err(e) => Err(anyhow!(format!("Logged in but failed to save token: {e}"))),
            }
        } else {
            Err(anyhow!(error_response::handle(response).await))
        }
    }

    pub async fn check(&self, backend_url: &Url) -> Result<String> {
        let token = self.store.load()?;

        let response = ClientBuilder::new()
            .build()?
            .get(backend_url.join("check")?)
            .bearer_auth(token)
            .send()
            .await
            .context("request failed")?;

        if response.status() == StatusCode::NO_CONTENT {
            Ok(String::from("Your token is valid"))
        } else {
            Err(anyhow!(error_response::handle(response).await))
        }
    }
}
