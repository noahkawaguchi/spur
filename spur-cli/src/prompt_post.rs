use crate::{format, request::RequestClient};
use anyhow::{Result, anyhow};
use reqwest::StatusCode;
use spur_shared::{requests::CreatePromptRequest, responses::SinglePromptResponse};
use validator::Validate;

pub struct PromptPostCommand<'a, C: RequestClient> {
    pub client: C,
    pub token: &'a str,
}

impl<C: RequestClient> PromptPostCommand<'_, C> {
    pub async fn new_prompt(&self, body: String) -> Result<String> {
        let req_body = CreatePromptRequest { body };
        req_body.validate()?;

        let response = self
            .client
            .post("prompts", req_body, Some(self.token))
            .await?;

        if response.status() == StatusCode::CREATED {
            let prompt = response.json::<SinglePromptResponse>().await?.prompt;
            Ok(format!("{prompt} successfully posted"))
        } else {
            Err(anyhow!(format::err_resp(response).await))
        }
    }

    pub async fn write_post(&self, prompt_id: i32) -> Result<String> {
        let prompt_response = self
            .client
            .get::<()>(&format!("prompts/{prompt_id}"), self.token, None)
            .await?;

        if prompt_response.status() != StatusCode::OK {
            return Err(anyhow!(format::err_resp(prompt_response).await));
        }

        let prompt = prompt_response.json::<SinglePromptResponse>().await?.prompt;

        // TODO: actually write a post here and make another request
        Ok(format!("PLACEHOLDER: write a post here for:\n    {prompt}"))
    }

    #[allow(clippy::unused_async)] // FIXME
    pub async fn profile(&self, username: Option<String>) -> Result<String> {
        Ok(format!(
            "PLACEHOLDER: get prompts and posts by {}",
            username.map_or_else(
                || String::from("the user"),
                |friend_username| friend_username
            )
        ))
    }

    #[allow(clippy::unused_async)] // FIXME
    pub async fn feed(&self) -> Result<String> {
        Ok(String::from(
            "PLACEHOLDER: get all friends' prompts and posts",
        ))
    }
}
