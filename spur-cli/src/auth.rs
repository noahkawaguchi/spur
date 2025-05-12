use crate::{error_response, input_validators, token_store};
use anyhow::{Context, Result};
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
        .await
        .context("request failed".red())?;

    if response.status() == StatusCode::CREATED {
        println!("{}", "Successfully registered".green());
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
        .await
        .context("request failed".red())?;

    if response.status() == StatusCode::OK {
        println!("{}", "Successfully logged in".green());
        token_store::save(&response.json::<LoginResponse>().await?.token)?;
        println!("{}", "Successfully saved token".green());
    } else {
        error_response::handle(response).await;
    }

    Ok(())
}

pub async fn check(backend_url: &Url) -> Result<()> {
    let token = token_store::load()?;

    let response = ClientBuilder::new()
        .build()?
        .get(backend_url.join("check")?)
        .bearer_auth(token)
        .send()
        .await
        .context("request failed".red())?;

    if response.status() == StatusCode::NO_CONTENT {
        println!("{}", "Your token is valid".green());
    } else {
        error_response::handle(response).await;
    }

    Ok(())
}
