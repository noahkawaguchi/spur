use colored::Colorize;
use reqwest::Response;
use spur_shared::dto::ErrorResponse;

/// Parses the body as an error response and prints the error message, or prints the status if the
/// parsing fails.
pub async fn handle(response: Response) {
    let status = response.status();

    println!(
        "{}",
        match response.json::<ErrorResponse>().await {
            Ok(err_resp) => err_resp.error,
            Err(_) => format!(
                "unexpected response from the server with status {}",
                status.canonical_reason().unwrap_or_else(|| status.as_str()),
            ),
        }
        .red(),
    );
}
