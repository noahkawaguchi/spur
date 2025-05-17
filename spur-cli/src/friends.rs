use crate::{error_response, request::RequestClient};
use anyhow::{Result, anyhow};
use reqwest::StatusCode;
use spur_shared::{
    requests::AddFriendRequest,
    responses::{SuccessResponse, UsernamesResponse},
};

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

    pub async fn list_friends(&self) -> Result<String> {
        let response = self.client.get("friends", self.token).await?;

        if response.status() == StatusCode::OK {
            let friends_list = response
                .json::<UsernamesResponse>()
                .await?
                .usernames
                .join("\n");

            Ok(friends_list)
        } else {
            Err(anyhow!(error_response::handle(response).await))
        }
    }

    pub async fn list_requests(&self) -> Result<String> {
        let response = self.client.get("requests", self.token).await?;

        if response.status() == StatusCode::OK {
            let requests_list = response
                .json::<UsernamesResponse>()
                .await?
                .usernames
                .join("\n");

            Ok(requests_list)
        } else {
            Err(anyhow!(error_response::handle(response).await))
        }
    }
}
