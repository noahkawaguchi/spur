use crate::{auth::AuthPrompt, input_validators};
use inquire::{Password, Text, error::InquireResult, validator::ValueRequiredValidator};
use spur_shared::dto::{LoginRequest, SignupRequest};

pub struct InteractiveAuthPrompt;

impl AuthPrompt for InteractiveAuthPrompt {
    fn signup(&self) -> InquireResult<SignupRequest> {
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

    fn login(&self) -> InquireResult<LoginRequest> {
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
}
