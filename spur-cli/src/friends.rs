use crate::{error_response, request::RequestClient};
use anyhow::{Result, anyhow};
use reqwest::StatusCode;
use spur_shared::{requests::AddFriendRequest, responses::SuccessResponse};

pub struct FriendsCommand<'a, C: RequestClient> {
    pub client: C,
    pub token: &'a str,
}

impl<C: RequestClient> FriendsCommand<'_, C> {
    pub async fn add_friend(&self, username: String) -> Result<String> {
        let body = AddFriendRequest { recipient_username: username };
        let response = self.client.post("add", body, Some(self.token)).await?;

        match response.status() {
            StatusCode::OK | StatusCode::CREATED => {
                Ok(response.json::<SuccessResponse>().await?.message)
            }
            _ => Err(anyhow!(error_response::handle(response).await)),
        }
    }
}
