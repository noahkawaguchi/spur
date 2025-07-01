use crate::{format, request::RequestClient};
use anyhow::{Result, anyhow};
use reqwest::StatusCode;
use spur_shared::{
    requests::AddFriendRequest,
    responses::{SuccessResponse, UsernamesResponse},
};
use validator::Validate;

pub struct FriendsCommand<'a, C: RequestClient> {
    pub client: C,
    pub token: &'a str,
}

impl<C: RequestClient> FriendsCommand<'_, C> {
    pub async fn add_friend(&self, username: String) -> Result<String> {
        let body = AddFriendRequest { recipient_username: username };
        body.validate()?;

        let response = self.client.post("friends", body, Some(self.token)).await?;
        match response.status() {
            StatusCode::OK | StatusCode::CREATED => {
                Ok(response.json::<SuccessResponse>().await?.message)
            }
            _ => Err(anyhow!(format::err_resp(response).await)),
        }
    }

    pub async fn list_friends(&self) -> Result<String> {
        let response = self.client.get("friends", self.token, None::<()>).await?;

        if response.status() == StatusCode::OK {
            let usernames = response.json::<UsernamesResponse>().await?.usernames;

            let usernames_list = format!(
                "Your friends:\n    {}",
                if usernames.is_empty() {
                    String::from("(no friends)")
                } else {
                    usernames.join("\n    ")
                }
            );

            Ok(usernames_list)
        } else {
            Err(anyhow!(format::err_resp(response).await))
        }
    }

    pub async fn list_requests(&self) -> Result<String> {
        let response = self
            .client
            .get("friends/requests", self.token, None::<()>)
            .await?;

        if response.status() == StatusCode::OK {
            let usernames = response.json::<UsernamesResponse>().await?.usernames;

            let usernames_list = format!(
                "Pending friend requests:\n    {}",
                if usernames.is_empty() {
                    String::from("(no requests)")
                } else {
                    format!(
                        "{}\n(use the `add` command to accept)",
                        usernames.join("\n    "),
                    )
                }
            );

            Ok(usernames_list)
        } else {
            Err(anyhow!(format::err_resp(response).await))
        }
    }
}
