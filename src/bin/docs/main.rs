use {spur::api::ApiDoc, std::error::Error, utoipa::OpenApi as _};

/// Prints the API documentation to stdout in (pretty) JSON format.
fn main() -> Result<(), Box<dyn Error>> {
    println!("{}", ApiDoc::openapi().to_pretty_json()?);
    Ok(())
}
