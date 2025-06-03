use crate::{format, request::RequestClient};
use anyhow::{Result, anyhow};
use reqwest::StatusCode;
use spur_shared::{
    requests::{CreatePromptRequest, UserContentParam},
    responses::{PromptsAndPostsResponse, SinglePostResponse, SinglePromptResponse},
};
use validator::Validate;

pub struct ContentCommand<'a, C: RequestClient> {
    pub client: C,
    pub token: &'a str,
}

impl<C: RequestClient> ContentCommand<'_, C> {
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

    pub async fn read_post(&self, post_id: i32) -> Result<String> {
        let response = self
            .client
            .get::<()>(&format!("posts/{post_id}"), self.token, None)
            .await?;

        if response.status() == StatusCode::OK {
            let post = response.json::<SinglePostResponse>().await?.post;
            Ok(format!(
                "PLACEHOLDER: Here is the post and its body for now:\n    {post}\n\n{}",
                post.body
            ))
        } else {
            Err(anyhow!(format::err_resp(response).await))
        }
    }

    pub async fn user_content(&self, username: Option<String>) -> Result<String> {
        let response = self
            .client
            .get(
                "content",
                self.token,
                Some(UserContentParam { author_username: username.clone() }),
            )
            .await?;

        if response.status() == StatusCode::OK {
            let content =
                format::pretty_content(&response.json::<PromptsAndPostsResponse>().await?);
            let author =
                username.map_or_else(|| String::from("you"), |friend_username| friend_username);

            Ok(format!("Prompts and posts by {author}:\n    {content}"))
        } else {
            Err(anyhow!(format::err_resp(response).await))
        }
    }

    pub async fn feed(&self) -> Result<String> {
        let response = self
            .client
            .get::<()>("content/friends", self.token, None)
            .await?;

        if response.status() == StatusCode::OK {
            let content =
                format::pretty_content(&response.json::<PromptsAndPostsResponse>().await?);
            Ok(format!("Prompts and posts by your friends:\n    {content}"))
        } else {
            Err(anyhow!(format::err_resp(response).await))
        }
    }
}
