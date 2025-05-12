use crate::input_validators;
use anyhow::Result;
use inquire::{Password, Text};
use reqwest::{ClientBuilder, StatusCode};
use spur_shared::dto::{ErrorResponse, SignupRequest};
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

    match response.status() {
        StatusCode::CREATED => println!("successfully registered"),
        status => match response.json::<ErrorResponse>().await {
            Ok(err_resp) => println!("{}", err_resp.error),
            Err(_) => println!(
                "unexpected response from the server with status {}",
                status.canonical_reason().unwrap_or(status.as_str()),
            ),
        },
    }

    Ok(())
}
