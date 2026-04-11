use spur::api::ApiDoc;
use std::error::Error;
use utoipa::OpenApi;

/// Prints the API documentation to stdout in (pretty) JSON format.
fn main() -> Result<(), Box<dyn Error>> {
    println!("{}", ApiDoc::openapi().to_pretty_json()?);
    Ok(())
}
