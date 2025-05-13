use crate::{
    handlers::auth_handlers::AuthService,
    models::{NewUser, User},
};
use anyhow::Result;
use spur_shared::dto::{LoginRequest, SignupRequest};

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    async fn insert_new(&self, new_user: &NewUser) -> sqlx::Result<()>;
    async fn get_by_email(&self, email: &str) -> sqlx::Result<User>;
    async fn get_by_username(&self, username: &str) -> sqlx::Result<User>;
}

#[derive(Clone)]
pub struct AuthSvc<R: UserRepository> {
    repo: R,
}

impl<R: UserRepository> AuthSvc<R> {
    pub const fn new(repo: R) -> Self { Self { repo } }
}

#[async_trait::async_trait]
impl<R: UserRepository> AuthService for AuthSvc<R> {
    /// Checks if an account with the given email or username already exists in the database.
    async fn email_username_available(&self, req: &SignupRequest) -> Result<(), String> {
        if self.repo.get_by_email(&req.email).await.is_ok() {
            return Err(String::from(
                "an account with the same email already exists",
            ));
        }

        if self.repo.get_by_username(&req.username).await.is_ok() {
            return Err(String::from(
                "an account with the same username already exists",
            ));
        }

        Ok(())
    }

    /// Hashes the password and creates a new user in the database.
    async fn register(&self, req: SignupRequest) -> Result<()> {
        let hashed = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)?;
        let new_user = NewUser::from_request(req, hashed);
        self.repo.insert_new(&new_user).await?;
        Ok(())
    }

    /// Checks `email` and `password` for a valid match in the database.
    async fn validate_credentials(&self, req: &LoginRequest) -> Result<User, String> {
        // Check if the user exists
        let Ok(user) = self.repo.get_by_email(&req.email).await else {
            return Err(String::from("invalid email"));
        };

        // Validate the password
        if !bcrypt::verify(&req.password, &user.password_hash).is_ok_and(|is_valid| is_valid) {
            return Err(String::from("invalid password"));
        }

        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod email_username_available {
        use super::*;
        use chrono::Utc;
        use mockall::predicate::*;

        fn new_alice() -> User {
            User {
                id: 1,
                name: String::from("Alice"),
                email: String::from("alice@example.com"),
                username: String::from("alice123"),
                password_hash: String::from("aeb451b%@!"),
                created_at: Utc::now(),
            }
        }

        #[tokio::test]
        async fn errors_for_existing_email() {
            let alice = new_alice();
            let alice_request = SignupRequest {
                name: String::from("New Alice"),
                email: alice.email.clone(),
                username: String::from("new_alice"),
                password: String::from("secret"),
            };

            let mut mock_repo = MockUserRepository::new();
            mock_repo.expect_insert_new().never();
            mock_repo.expect_get_by_username().never();
            mock_repo
                .expect_get_by_email()
                .with(eq(alice_request.email.clone()))
                .once()
                .return_once(|_| Ok(alice));

            let auth_svc = AuthSvc::new(mock_repo);

            assert_eq!(
                auth_svc.email_username_available(&alice_request).await,
                Err(String::from(
                    "an account with the same email already exists",
                )),
            );
        }

        #[tokio::test]
        async fn errors_for_existing_username() {
            let alice = new_alice();
            let alice_request = SignupRequest {
                name: String::from("New Alice"),
                email: String::from("new_alice@example.com"),
                username: alice.username.clone(),
                password: String::from("super secret"),
            };

            let mut mock_repo = MockUserRepository::new();
            mock_repo.expect_insert_new().never();
            mock_repo
                .expect_get_by_email()
                .with(eq(alice_request.email.clone()))
                .once()
                .return_once(|_| Err(sqlx::Error::RowNotFound));
            mock_repo
                .expect_get_by_username()
                .with(eq(alice_request.username.clone()))
                .once()
                .return_once(|_| Ok(alice));

            let auth_svc = AuthSvc::new(mock_repo);

            assert_eq!(
                auth_svc.email_username_available(&alice_request).await,
                Err(String::from(
                    "an account with the same username already exists",
                )),
            );
        }

        #[tokio::test]
        async fn returns_ok_if_available() {
            let alice_request = SignupRequest {
                name: String::from("Unique Alice"),
                email: String::from("unique@alice.com"),
                username: String::from("alice_the_unique"),
                password: String::from("maximum security"),
            };

            let mut mock_repo = MockUserRepository::new();
            mock_repo.expect_insert_new().never();
            mock_repo
                .expect_get_by_email()
                .with(eq(alice_request.email.clone()))
                .once()
                .return_once(|_| Err(sqlx::Error::RowNotFound));
            mock_repo
                .expect_get_by_username()
                .with(eq(alice_request.username.clone()))
                .once()
                .return_once(|_| Err(sqlx::Error::RowNotFound));

            let auth_svc = AuthSvc::new(mock_repo);

            assert_eq!(
                auth_svc.email_username_available(&alice_request).await,
                Ok(()),
            );
        }
    }
}
