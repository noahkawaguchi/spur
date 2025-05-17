use std::sync::Arc;

use crate::{
    handlers::auth_handlers::Authenticator,
    models::user::{NewUser, User},
    repositories::user_repo::UserStore,
};
use anyhow::Result;
use spur_shared::requests::{LoginRequest, SignupRequest};

#[derive(Clone)]
pub struct AuthSvc {
    store: Arc<dyn UserStore>,
}

impl AuthSvc {
    pub const fn new(store: Arc<dyn UserStore>) -> Self { Self { store } }
}

#[async_trait::async_trait]
impl Authenticator for AuthSvc {
    async fn email_username_available(&self, req: &SignupRequest) -> Result<(), String> {
        if self.store.get_by_email(&req.email).await.is_ok() {
            return Err(String::from(
                "an account with the same email already exists",
            ));
        }

        if self.store.get_by_username(&req.username).await.is_ok() {
            return Err(String::from(
                "an account with the same username already exists",
            ));
        }

        Ok(())
    }

    async fn register(&self, req: SignupRequest) -> Result<()> {
        let hashed = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)?;
        let new_user = NewUser::from_request(req, hashed);
        self.store.insert_new(&new_user).await?;
        Ok(())
    }

    async fn validate_credentials(&self, req: &LoginRequest) -> Result<User, String> {
        // Check if the user exists
        let Ok(user) = self.store.get_by_email(&req.email).await else {
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
    use crate::repositories::user_repo::MockUserStore;
    use chrono::Utc;
    use mockall::predicate::eq;

    mod email_username_available {
        use super::*;

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

            let mut mock_repo = MockUserStore::new();
            mock_repo.expect_insert_new().never();
            mock_repo.expect_get_by_username().never();
            mock_repo
                .expect_get_by_email()
                .with(eq(alice_request.email.clone()))
                .once()
                .return_once(|_| Ok(alice));

            let auth_svc = AuthSvc::new(Arc::new(mock_repo));

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

            let mut mock_repo = MockUserStore::new();
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

            let auth_svc = AuthSvc::new(Arc::new(mock_repo));

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

            let mut mock_repo = MockUserStore::new();
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

            let auth_svc = AuthSvc::new(Arc::new(mock_repo));

            assert_eq!(
                auth_svc.email_username_available(&alice_request).await,
                Ok(()),
            );
        }
    }

    mod register {
        use super::*;

        #[tokio::test]
        async fn correctly_creates_new_user_from_request() {
            let (name, email, username, password) = (
                String::from("Alice New"),
                String::from("alice@new.you"),
                String::from("alice_new"),
                String::from("top secret"),
            );

            let alice_request = SignupRequest {
                name: name.clone(),
                email: email.clone(),
                username: username.clone(),
                password: password.clone(),
            };

            let mut mock_repo = MockUserStore::new();

            mock_repo.expect_get_by_email().never();
            mock_repo.expect_get_by_username().never();
            mock_repo
                .expect_insert_new()
                .withf(move |user| {
                    user.name == name
                        && user.email == email
                        && user.username == username
                        && bcrypt::verify(&password, &user.password_hash)
                            .expect("failed to verify password hash")
                })
                .once()
                .return_once(|_| Ok(()));

            let auth_svc = AuthSvc::new(Arc::new(mock_repo));

            assert!(matches!(
                auth_svc.register(alice_request).await,
                anyhow::Result::Ok(()),
            ));
        }
    }

    mod validate_credentials {
        use super::*;

        #[tokio::test]
        async fn errors_for_invalid_email() {
            let login_request = LoginRequest {
                email: String::from("bob@bob.bob"),
                password: String::from("extremely secure"),
            };

            let mut mock_repo = MockUserStore::new();
            mock_repo.expect_insert_new().never();
            mock_repo.expect_get_by_username().never();
            mock_repo
                .expect_get_by_email()
                .with(eq(login_request.email.clone()))
                .once()
                .return_once(|_| Err(sqlx::Error::RowNotFound));

            let auth_svc = AuthSvc::new(Arc::new(mock_repo));

            assert_eq!(
                auth_svc.validate_credentials(&login_request).await,
                Err(String::from("invalid email")),
            );
        }

        #[tokio::test]
        async fn errors_for_invalid_password() {
            let correct_bob = User {
                id: 42,
                name: String::from("Bob"),
                email: String::from("bob@email.co.uk"),
                username: String::from("bobby_bob"),
                password_hash: bcrypt::hash("correct password", bcrypt::DEFAULT_COST)
                    .expect("failed to hash password"),
                created_at: Utc::now(),
            };

            let incorrect_request = LoginRequest {
                email: correct_bob.email.clone(),
                password: String::from("incorrect password"),
            };

            let mut mock_repo = MockUserStore::new();
            mock_repo.expect_insert_new().never();
            mock_repo.expect_get_by_username().never();
            mock_repo
                .expect_get_by_email()
                .with(eq(incorrect_request.email.clone()))
                .once()
                .return_once(|_| Ok(correct_bob));

            let auth_svc = AuthSvc::new(Arc::new(mock_repo));

            assert_eq!(
                auth_svc.validate_credentials(&incorrect_request).await,
                Err(String::from("invalid password")),
            );
        }

        #[tokio::test]
        async fn returns_user_for_valid_credentials() {
            let password = String::from("correct password");

            let correct_bob = User {
                id: 42,
                name: String::from("Bob"),
                email: String::from("bob@email.co.uk"),
                username: String::from("bobby_bob"),
                password_hash: bcrypt::hash(&password, bcrypt::DEFAULT_COST)
                    .expect("failed to hash password"),
                created_at: Utc::now(),
            };
            let also_bob = correct_bob.clone();

            let correct_request = LoginRequest { email: correct_bob.email.clone(), password };

            let mut mock_repo = MockUserStore::new();
            mock_repo.expect_insert_new().never();
            mock_repo.expect_get_by_username().never();
            mock_repo
                .expect_get_by_email()
                .with(eq(correct_request.email.clone()))
                .once()
                .return_once(|_| Ok(also_bob));

            let auth_svc = AuthSvc::new(Arc::new(mock_repo));

            assert_eq!(
                auth_svc.validate_credentials(&correct_request).await,
                Ok(correct_bob),
            );
        }
    }
}
