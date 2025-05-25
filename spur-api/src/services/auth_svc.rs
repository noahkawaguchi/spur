use super::domain_error::{AuthError, DomainError};
use crate::{
    handlers::auth_handlers::Authenticator,
    models::user::{User, UserRegistration},
    repositories::{insertion_error::InsertionError, user_repo::UserStore},
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
    async fn register(&self, reg: UserRegistration) -> Result<(), DomainError> {
        let pw_hash =
            bcrypt::hash(&reg.password, bcrypt::DEFAULT_COST).map_err(TechnicalError::from)?;

        match self
            .store
            .insert_new(&reg.into_new_user_with(pw_hash))
            .await
        {
            Ok(()) => Ok(()),
            Err(InsertionError::UniqueViolation(v)) if v.contains("email") => {
                Err(AuthError::DuplicateEmail.into())
            }
            Err(InsertionError::UniqueViolation(v)) if v.contains("username") => {
                Err(AuthError::DuplicateUsername.into())
            }
            Err(InsertionError::UniqueViolation(v)) => {
                Err(TechnicalError::Unexpected(format!("Unexpected unique violation: {v}")).into())
            }
            Err(InsertionError::Technical(e)) => Err(TechnicalError::Database(e).into()),
        }
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

    mod register {
        use super::*;

        fn alice_registration() -> UserRegistration {
            UserRegistration {
                name: String::from("New Alice"),
                email: String::from("alice_new@example.com"),
                username: String::from("new_alice"),
                password: String::from("secret"),
            }
        }

        #[tokio::test]
        async fn errors_for_existing_email() {
            let alice_reg = alice_registration();
            let alice_reg_clone = alice_reg.clone();
            let pw_clone = alice_reg.password.clone();

            let mut mock_repo = MockUserStore::new();
            mock_repo
                .expect_insert_new()
                .withf(move |u| {
                    u.name == alice_reg_clone.name
                        && u.email == alice_reg_clone.email
                        && u.username == alice_reg_clone.username
                        && bcrypt::verify(&pw_clone, &u.password_hash)
                            .expect("failed to verify password hash")
                })
                .once()
                .return_once(|_| {
                    Err(InsertionError::UniqueViolation(String::from(
                        "users_email_unique",
                    )))
                });

            let auth_svc = AuthSvc::new(Arc::new(mock_repo));
            let result = auth_svc.register(alice_reg).await;

            assert!(matches!(
                result,
                Err(DomainError::Auth(AuthError::DuplicateEmail)),
            ));
        }

        #[tokio::test]
        async fn errors_for_existing_username() {
            let alice_reg = alice_registration();
            let alice_reg_clone = alice_reg.clone();
            let pw_clone = alice_reg.password.clone();

            let mut mock_repo = MockUserStore::new();
            mock_repo
                .expect_insert_new()
                .withf(move |u| {
                    u.name == alice_reg_clone.name
                        && u.email == alice_reg_clone.email
                        && u.username == alice_reg_clone.username
                        && bcrypt::verify(&pw_clone, &u.password_hash)
                            .expect("failed to verify password hash")
                })
                .once()
                .return_once(|_| {
                    Err(InsertionError::UniqueViolation(String::from(
                        "users_username_unique",
                    )))
                });

            let auth_svc = AuthSvc::new(Arc::new(mock_repo));
            let result = auth_svc.register(alice_reg).await;

            assert!(matches!(
                result,
                Err(DomainError::Auth(AuthError::DuplicateUsername)),
            ));
        }

        #[tokio::test]
        async fn correctly_creates_new_user_from_request() {
            let alice_reg = alice_registration();

            let (name, email, username, password) = (
                alice_reg.name.clone(),
                alice_reg.email.clone(),
                alice_reg.username.clone(),
                alice_reg.password.clone(),
            );

            let mut mock_repo = MockUserStore::new();
            mock_repo
                .expect_insert_new()
                .withf(move |u| {
                    u.name == name
                        && u.email == email
                        && u.username == username
                        && bcrypt::verify(&password, &u.password_hash)
                            .expect("failed to verify password hash")
                })
                .once()
                .return_once(|_| Ok(()));

            let auth_svc = AuthSvc::new(Arc::new(mock_repo));

            assert!(matches!(
                auth_svc.register(alice_reg).await,
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
