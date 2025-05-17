use chrono::{DateTime, Utc};
use spur_shared::requests::SignupRequest;

#[cfg_attr(test, derive(Debug, Clone, PartialEq, Eq))]
pub struct NewUser {
    pub name: String,
    pub email: String,
    pub username: String,
    pub password_hash: String,
}

impl NewUser {
    pub fn from_request(req: SignupRequest, password_hash: String) -> Self {
        Self { name: req.name, email: req.email, username: req.username, password_hash }
    }
}

#[cfg_attr(test, derive(Debug, Clone, PartialEq, Eq))]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
impl PartialEq<&NewUser> for User {
    fn eq(&self, other: &&NewUser) -> bool {
        self.name == other.name
            && self.email == other.email
            && self.username == other.username
            && self.password_hash == other.password_hash
    }
}
