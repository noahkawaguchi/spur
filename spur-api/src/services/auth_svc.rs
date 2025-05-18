use super::domain_error::{AuthError, DomainError};
use crate::{
    handlers::auth_handlers::Authenticator,
    models::user::{User, UserRegistration},
    repositories::user_repo::UserStore,
    technical_error::TechnicalError,
};
use anyhow::Result;
use std::sync::Arc;

#[derive(Clone)]
pub struct AuthSvc {
    store: Arc<dyn UserStore>,
}

impl AuthSvc {
    pub const fn new(store: Arc<dyn UserStore>) -> Self { Self { store } }
}

#[async_trait::async_trait]
impl Authenticator for AuthSvc {
    async fn email_username_available(
        &self,
        email: &str,
        username: &str,
    ) -> Result<(), DomainError> {
        // Attempting to get a user with the provided email/username will return None if the
        // email/username is available.
        self.store
            .get_by_email(email)
            .await?
            .map_or(Ok(()), |_| Err(AuthError::DuplicateEmail))?;

        self.store
            .get_by_username(username)
            .await?
            .map_or(Ok(()), |_| Err(AuthError::DuplicateUsername))?;

        Ok(())
    }

    async fn register(&self, reg: UserRegistration) -> Result<(), DomainError> {
        let pw_hash =
            bcrypt::hash(&reg.password, bcrypt::DEFAULT_COST).map_err(TechnicalError::from)?;

        self.store
            .insert_new(&reg.into_new_user_with(pw_hash))
            .await?;

        Ok(())
    }

    async fn validate_credentials(&self, email: &str, password: &str) -> Result<User, DomainError> {
        // Validate the email and get the associated user
        let user = self
            .store
            .get_by_email(email)
            .await?
            .ok_or(AuthError::InvalidEmail)?;

        // Validate the password
        bcrypt::verify(password, &user.password_hash)
            .map_err(TechnicalError::from)?
            .then_some(())
            .ok_or(AuthError::InvalidPassword)?;

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
            let alice_registration = UserRegistration {
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
                .with(eq(alice_registration.email.clone()))
                .once()
                .return_once(|_| Ok(Some(alice)));

            let auth_svc = AuthSvc::new(Arc::new(mock_repo));
            let result = auth_svc
                .email_username_available(&alice_registration.email, &alice_registration.password)
                .await;

            assert!(matches!(
                result,
                Err(DomainError::Auth(AuthError::DuplicateEmail)),
            ));
        }

        #[tokio::test]
        async fn errors_for_existing_username() {
            let alice = new_alice();
            let alice_registration = UserRegistration {
                name: String::from("New Alice"),
                email: String::from("new_alice@example.com"),
                username: alice.username.clone(),
                password: String::from("super secret"),
            };

            let mut mock_repo = MockUserStore::new();
            mock_repo.expect_insert_new().never();
            mock_repo
                .expect_get_by_email()
                .with(eq(alice_registration.email.clone()))
                .once()
                .return_once(|_| Ok(None));
            mock_repo
                .expect_get_by_username()
                .with(eq(alice_registration.username.clone()))
                .once()
                .return_once(|_| Ok(Some(alice)));

            let auth_svc = AuthSvc::new(Arc::new(mock_repo));
            let result = auth_svc
                .email_username_available(&alice_registration.email, &alice_registration.username)
                .await;

            assert!(matches!(
                result,
                Err(DomainError::Auth(AuthError::DuplicateUsername)),
            ));
        }

        #[tokio::test]
        async fn returns_ok_if_available() {
            let alice_registration = UserRegistration {
                name: String::from("Unique Alice"),
                email: String::from("unique@alice.com"),
                username: String::from("alice_the_unique"),
                password: String::from("maximum security"),
            };

            let mut mock_repo = MockUserStore::new();
            mock_repo.expect_insert_new().never();
            mock_repo
                .expect_get_by_email()
                .with(eq(alice_registration.email.clone()))
                .once()
                .return_once(|_| Ok(None));
            mock_repo
                .expect_get_by_username()
                .with(eq(alice_registration.username.clone()))
                .once()
                .return_once(|_| Ok(None));

            let auth_svc = AuthSvc::new(Arc::new(mock_repo));

            assert!(matches!(
                auth_svc
                    .email_username_available(
                        &alice_registration.email,
                        &alice_registration.username,
                    )
                    .await,
                Ok(()),
            ));
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

            let alice_registration = UserRegistration {
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
                auth_svc.register(alice_registration).await,
                anyhow::Result::Ok(()),
            ));
        }
    }

    mod validate_credentials {
        use super::*;

        #[tokio::test]
        async fn errors_for_invalid_email() {
            let email = String::from("bob@bob.bob");
            let password = String::from("extremely secure");

            let mut mock_repo = MockUserStore::new();
            mock_repo.expect_insert_new().never();
            mock_repo.expect_get_by_username().never();
            mock_repo
                .expect_get_by_email()
                .with(eq(email.clone()))
                .once()
                .return_once(|_| Ok(None));

            let auth_svc = AuthSvc::new(Arc::new(mock_repo));
            let result = auth_svc.validate_credentials(&email, &password).await;

            assert!(matches!(
                result,
                Err(DomainError::Auth(AuthError::InvalidEmail))
            ));
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

            let correct_email = correct_bob.email.clone();
            let incorrect_password = String::from("incorrect password");

            let mut mock_repo = MockUserStore::new();
            mock_repo.expect_insert_new().never();
            mock_repo.expect_get_by_username().never();
            mock_repo
                .expect_get_by_email()
                .with(eq(correct_email.clone()))
                .once()
                .return_once(|_| Ok(Some(correct_bob)));

            let auth_svc = AuthSvc::new(Arc::new(mock_repo));
            let result = auth_svc
                .validate_credentials(&correct_email, &incorrect_password)
                .await;

            assert!(matches!(
                result,
                Err(DomainError::Auth(AuthError::InvalidPassword))
            ));
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

            let mut mock_repo = MockUserStore::new();
            mock_repo.expect_insert_new().never();
            mock_repo.expect_get_by_username().never();
            mock_repo
                .expect_get_by_email()
                .with(eq(correct_bob.email.clone()))
                .once()
                .return_once(|_| Ok(Some(also_bob)));

            let auth_svc = AuthSvc::new(Arc::new(mock_repo));
            let result = auth_svc
                .validate_credentials(&correct_bob.email, &password)
                .await;

            match result {
                Ok(user) => assert_eq!(user, correct_bob),
                other => panic!("unexpected result: {other:?}"),
            }
        }
    }
}
