pub mod router;

mod dto;
mod error;
mod handler;
mod middleware;
mod validated_json;

mod utoipa_security {
    use utoipa::{
        Modify,
        openapi::{
            OpenApi,
            security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
        },
    };

    pub struct JwtAddon;

    impl Modify for JwtAddon {
        fn modify(&self, openapi: &mut OpenApi) {
            if let Some(components) = openapi.components.as_mut() {
                components.add_security_scheme(
                    "jwt",
                    SecurityScheme::Http(
                        HttpBuilder::new()
                            .scheme(HttpAuthScheme::Bearer)
                            .bearer_format("JWT")
                            .description(Some("Enter the token created at signup or login"))
                            .build(),
                    ),
                );
            }
        }
    }
}
