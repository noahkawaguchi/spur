use crate::{error_response, request::RequestClient, token_store::TokenStore};
use anyhow::{Result, anyhow};
use reqwest::StatusCode;
use spur_shared::{requests::AddFriendRequest, responses::SuccessResponse};
use std::sync::Arc;

pub struct FriendsCommand<C: RequestClient> {
    pub client: C,
    pub store: Arc<dyn TokenStore>,
}

impl<C: RequestClient> FriendsCommand<C> {
    pub async fn add_friend(&self, username: String) -> Result<String> {
        let token = self.store.load()?;
        let body = AddFriendRequest { recipient_username: username };

        let response = self.client.post("add", body, Some(&token)).await?;

        match response.status() {
            StatusCode::OK | StatusCode::CREATED => {
                Ok(response.json::<SuccessResponse>().await?.message)
            }
            _ => Err(anyhow!(error_response::handle(response).await)),
        }
    }
}
