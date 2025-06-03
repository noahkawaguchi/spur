use crate::input_validators;
use anyhow::Result;
use colored::Colorize;
use inquire::{Editor, Password, Text, error::InquireResult, validator::ValueRequiredValidator};
use spur_shared::requests::{LoginRequest, SignupRequest};
use std::fmt::Display;

/// Prompts the user for name, email, username, and password.
pub fn signup() -> InquireResult<SignupRequest> {
    let name = Text::new("Name:")
        .with_validator(ValueRequiredValidator::new("Name cannot be empty"))
        .prompt()?;

    let email = Text::new("Email:")
        .with_validator(input_validators::email)
        .prompt()?;

    let username = Text::new("Username:")
        .with_validator(ValueRequiredValidator::new("Username cannot be empty"))
        .prompt()?;

    let password = Password::new("Password:")
        .with_validator(input_validators::password)
        .with_formatter(&|_| String::from("[hidden]"))
        .prompt()?;

    Ok(SignupRequest { name, email, username, password })
}

/// Prompts the user for email and password.
pub fn login() -> InquireResult<LoginRequest> {
    let email = Text::new("Email:")
        .with_validator(input_validators::email)
        .prompt()?;

    // For logging into an existing account, only ask for the password once and don't check
    // password requirements other than being non-empty
    let password = Password::new("Password:")
        .with_validator(ValueRequiredValidator::new("Password cannot be empty"))
        .without_confirmation()
        .with_formatter(&|_| String::from("[hidden]"))
        .prompt()?;

    Ok(LoginRequest { email, password })
}

/// Prompts the user for the body of a post via a text editor.
pub fn post_body(message: impl Display, editor: Option<&str>) -> Result<String> {
    let prompt_line = format!("{}\n", message.to_string().bright_magenta());

    let mut prompt = Editor::new(&prompt_line)
        .with_validator(ValueRequiredValidator::new("Post cannot be empty"))
        .with_formatter(&|_| String::from("post body received"));

    if let Some(custom_editor) = editor {
        prompt = prompt.with_editor_command(custom_editor.as_ref());
    }

    Ok(prompt.prompt()?)
}
