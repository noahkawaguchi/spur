use crate::{error_response, input_validators};
use anyhow::Result;
use colored::Colorize;
use inquire::{Password, Text};
use reqwest::{ClientBuilder, StatusCode};
use spur_shared::dto::{LoginRequest, LoginResponse, SignupRequest};
use url::Url;

pub async fn signup(backend_url: &Url) -> Result<()> {
    let name = Text::new("Name:")
        .with_validator(input_validators::nonempty)
        .prompt()?;

    let email = Text::new("Email:")
        .with_validator(input_validators::email)
        .prompt()?;

    let username = Text::new("Username:")
        .with_validator(input_validators::nonempty)
        .prompt()?;

    let password = Password::new("Password:")
        .with_formatter(&|_| String::from("[hidden]"))
        .with_validator(input_validators::password)
        .prompt()?;

    let body = SignupRequest { name, email, username, password };

    let response = ClientBuilder::new()
        .build()?
        .post(backend_url.join("signup")?)
        .json(&body)
        .send()
        .await?;

    if response.status() == StatusCode::CREATED {
        println!("{}", "successfully registered".green());
    } else {
        error_response::handle(response).await;
    }

    Ok(())
}

pub async fn login(backend_url: &Url) -> Result<()> {
    let email = Text::new("Email:")
        .with_validator(input_validators::email)
        .prompt()?;

    // For logging into an existing account, only ask for the password once and don't check
    // password requirements other than being non-empty
    let password = Password::new("Password:")
        .with_formatter(&|_| String::from("[hidden]"))
        .with_validator(input_validators::nonempty)
        .without_confirmation()
        .prompt()?;

    let body = LoginRequest { email, password };

    let response = ClientBuilder::new()
        .build()?
        .post(backend_url.join("login")?)
        .json(&body)
        .send()
        .await?;

    if response.status() == StatusCode::OK {
        println!(
            "{}",
            format!(
                "need to save this token: {}", // TODO
                response.json::<LoginResponse>().await?.token,
            )
            .green()
        );
    } else {
        error_response::handle(response).await;
    }

    Ok(())
}
