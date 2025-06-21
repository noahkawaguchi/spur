use crate::{format, interactive, request::RequestClient, token_store::TokenStore};
use anyhow::{Result, anyhow};
use reqwest::StatusCode;
use spur_shared::responses::LoginResponse;

pub struct AuthCommand<S, C>
where
    S: TokenStore,
    C: RequestClient,
{
    pub store: S,
    pub client: C,
}

impl<S, C> AuthCommand<S, C>
where
    S: TokenStore,
    C: RequestClient,
{
    pub async fn signup(&self) -> Result<String> {
        let body = interactive::signup()?;
        let response = self.client.post("auth/signup", body, None).await?;

        if response.status() == StatusCode::CREATED {
            Ok(String::from("Successfully registered"))
        } else {
            Err(anyhow!(format::err_resp(response).await))
        }
    }

    pub async fn login(&self) -> Result<String> {
        let body = interactive::login()?;
        let response = self.client.post("auth/login", body, None).await?;

        if response.status() == StatusCode::OK {
            match self
                .store
                .save(&response.json::<LoginResponse>().await?.token)
            {
                Ok(()) => Ok(String::from("Successfully logged in and saved token")),
                Err(e) => Err(anyhow!(format!("Logged in but failed to save token: {e}"))),
            }
        } else {
            Err(anyhow!(format::err_resp(response).await))
        }
    }

    pub async fn check(&self) -> Result<String> {
        let token = self.store.load()?;
        let response = self.client.get("auth/check", &token, None::<()>).await?;

        if response.status() == StatusCode::NO_CONTENT {
            Ok(String::from("Your token is valid"))
        } else {
            Err(anyhow!(format::err_resp(response).await))
        }
    }
}
