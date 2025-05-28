use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid password")]
    InvalidPassword,

    #[error("Expired or invalid token. Try logging in again.")]
    JwtValidation,
}
