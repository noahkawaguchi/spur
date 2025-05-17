use reqwest::Response;
use spur_shared::responses::ErrorResponse;

/// Parses the body as an error response and returns the error message, or the status if the
/// parsing fails.
pub async fn handle(response: Response) -> String {
    let status = response.status();

    match response.json::<ErrorResponse>().await {
        Ok(err_resp) => err_resp.error,
        Err(_) => format!(
            "unexpected response from the server with status {}",
            status.canonical_reason().unwrap_or_else(|| status.as_str()),
        ),
    }
}
