pub mod requests;
pub mod responses;
pub mod signup_request;

#[cfg(test)]
pub mod dummy_data {
    use super::{requests::LoginRequest, signup_request::SignupRequest};

    pub fn dummy_signup_request() -> SignupRequest {
        SignupRequest {
            name: String::from("Christina Ani-Tsi RHC"),
            email: String::from("name@backwards.moc"),
            username: String::from("chris_and_tina"),
            password: String::from("2shh!5hh#H"),
        }
    }

    pub fn dummy_login_request() -> LoginRequest {
        LoginRequest {
            email: String::from("name@backwards.moc"),
            password: String::from("2shh!5hh#H"),
        }
    }
}
