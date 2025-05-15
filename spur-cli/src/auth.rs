use crate::{error_response, request::RequestClient};
use anyhow::{Result, anyhow};
use inquire::error::InquireResult;
use reqwest::StatusCode;
use spur_shared::dto::{LoginRequest, LoginResponse, SignupRequest};

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

pub struct AuthCommand<P, S, C>
where
    P: AuthPrompt,
    S: TokenStore,
    C: RequestClient,
{
    pub prompt: P,
    pub store: S,
    pub client: C,
}

impl<P, S, C> AuthCommand<P, S, C>
where
    P: AuthPrompt,
    S: TokenStore,
    C: RequestClient,
{
    pub async fn signup(&self) -> Result<String> {
        let body = self.prompt.signup()?;
        let response = self.client.post("signup", body).await?;

        if response.status() == StatusCode::CREATED {
            Ok(String::from("Successfully registered"))
        } else {
            Err(anyhow!(error_response::handle(response).await))
        }
    }

    pub async fn login(&self) -> Result<String> {
        let body = self.prompt.login()?;
        let response = self.client.post("login", body).await?;

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

    pub async fn check(&self) -> Result<String> {
        let token = self.store.load()?;
        let response = self.client.get("check", &token).await?;

        if response.status() == StatusCode::NO_CONTENT {
            Ok(String::from("Your token is valid"))
        } else {
            Err(anyhow!(error_response::handle(response).await))
        }
    }
}
