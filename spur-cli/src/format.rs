use anyhow::{Error, Result, anyhow};
use colored::{ColoredString, Colorize};
use reqwest::Response;
use spur_shared::responses::ErrorResponse;

/// Parses the body as an error response and returns the error message, or the status if the
/// parsing fails.
pub async fn err_resp(response: Response) -> String {
    let status = response.status();

    match response.json::<ErrorResponse>().await {
        Ok(err_resp) => err_resp.error,
        Err(_) => format!(
            "unexpected response from the server with status {}",
            status.canonical_reason().unwrap_or_else(|| status.as_str()),
        ),
    }
}

/// Colors the first (or only) line of a success message green and the first (or only) line of an
/// error message red.
pub fn color_first_line(result: Result<String, Error>) -> Result<ColoredString, Error> {
    match result {
        Ok(message) => match message.split_once('\n') {
            Some((first, rest)) => Ok(format!("{}\n{}", first.green(), rest).into()),
            None => Ok(message.green()),
        },
        Err(e) => {
            let err = e.to_string();
            match err.split_once('\n') {
                Some((first, rest)) => Err(anyhow!(format!("{}\n{}", first.red(), rest))),
                None => Err(anyhow!(err.red())),
            }
        }
    }
}
