mod claims;
pub mod service;

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid password")]
    InvalidPassword,

    #[error("Expired or invalid token. Try logging in again.")]
    JwtValidation,

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}
