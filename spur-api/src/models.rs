use chrono::{DateTime, Utc};
use spur_shared::dto::SignupRequest;

pub struct NewUser<'a> {
    pub name: &'a str,
    pub email: &'a str,
    pub username: &'a str,
    pub password_hash: &'a str,
}

impl<'a> NewUser<'a> {
    pub fn from_request(req: &'a SignupRequest, password_hash: &'a str) -> Self {
        Self { name: &req.name, email: &req.email, username: &req.username, password_hash }
    }
}

pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}
