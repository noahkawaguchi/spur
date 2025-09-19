use crate::{
    app_services::{Authenticator, uow::UnitOfWork},
    domain::{
        auth::{AuthError, AuthProvider},
        user::UserRepo,
    },
    models::user::UserRegistration,
};

pub struct AuthenticatorSvc<U, R, P> {
    uow: U,
    repo: R,
    provider: P,
}

impl<U, R, P> AuthenticatorSvc<U, R, P> {
    pub const fn new(uow: U, repo: R, provider: P) -> Self { Self { uow, repo, provider } }
}

#[async_trait::async_trait]
impl<U, R, P> Authenticator for AuthenticatorSvc<U, R, P>
where
    U: UnitOfWork,
    R: UserRepo,
    P: AuthProvider,
{
    async fn signup(&self, reg: UserRegistration) -> Result<String, AuthError> {
        let pw_hash = self.provider.hash_pw(&reg.password)?;
        let new_user = reg.into_new_user_with_hash(pw_hash);

        let user_id = self
            .repo
            .insert_new(self.uow.single_exec(), &new_user)
            .await?
            .id;

        self.provider.create_token(user_id).map_err(Into::into)
    }

    async fn login(&self, email: &str, pw: &str) -> Result<String, AuthError> {
        let existing_user = self
            .repo
            .get_by_email(self.uow.single_exec(), email)
            .await?
            .ok_or(AuthError::NotFound)?;

        self.provider
            .is_valid_pw(pw, &existing_user.password_hash)?
            .then(|| self.provider.create_token(existing_user.id))
            .ok_or(AuthError::InvalidPassword)?
            .map_err(Into::into)
    }

    fn validate_token(&self, token: &str) -> Result<i32, AuthError> {
        self.provider
            .validate_token(token)
            .map_err(|_| AuthError::TokenValidation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::{RepoError, auth::MockAuthProvider},
        models::user::User,
        test_utils::{fake_db::fake_pool, mock_repos::MockUserRepo},
    };
    use chrono::Utc;
    use mockall::predicate::eq;

    fn alice_user() -> User {
        User {
            id: 62,
            name: String::from("Alice Palace"),
            email: String::from("alice@palace.org"),
            username: String::from("al_is"),
            password_hash: String::from("ab%#S"),
            created_at: Utc::now(),
        }
    }

    fn alice_registration() -> UserRegistration {
        UserRegistration {
            name: String::from("Alice Palace"),
            email: String::from("alice@palace.org"),
            username: String::from("al_is"),
            password: String::from("super-secure"),
        }
    }

    mod signup {
        use super::*;

        #[tokio::test]
        async fn errors_for_existing_email() {
            let alice = alice_registration();
            let alice_clone = alice.clone();
            let pw_hash = "ab43$@baf$$CO";

            let mut mock_provider = MockAuthProvider::new();
            mock_provider
                .expect_hash_pw()
                .with(eq(alice_clone.password))
                .once()
                .return_once(|_| Ok(pw_hash.to_string()));
            mock_provider.expect_create_token().never();

            let mock_repo = MockUserRepo {
                insert_new: Some(Box::new(move |u| {
                    assert!(
                        alice_clone.name == u.name
                            && alice_clone.email == u.email
                            && alice_clone.username == u.username
                            && pw_hash == u.password_hash
                    );
                    Err(RepoError::UniqueViolation(String::from(
                        "users_email_unique",
                    )))
                })),
                ..Default::default()
            };

            let auth = AuthenticatorSvc::new(fake_pool(), mock_repo, mock_provider);
            let result = auth.signup(alice).await;

            assert!(matches!(result, Err(AuthError::DuplicateEmail)));
        }

        #[tokio::test]
        async fn errors_for_existing_username() {
            let alice = alice_registration();
            let alice_clone = alice.clone();
            let pw_hash = "ab43$@baf$$CO";

            let mut mock_provider = MockAuthProvider::new();
            mock_provider
                .expect_hash_pw()
                .with(eq(alice_clone.password))
                .once()
                .return_once(|_| Ok(pw_hash.to_string()));
            mock_provider.expect_create_token().never();

            let mock_repo = MockUserRepo {
                insert_new: Some(Box::new(move |u| {
                    assert!(
                        alice_clone.name == u.name
                            && alice_clone.email == u.email
                            && alice_clone.username == u.username
                            && pw_hash == u.password_hash
                    );
                    Err(RepoError::UniqueViolation(String::from(
                        "users_username_unique",
                    )))
                })),
                ..Default::default()
            };

            let auth = AuthenticatorSvc::new(fake_pool(), mock_repo, mock_provider);
            let result = auth.signup(alice).await;

            assert!(matches!(result, Err(AuthError::DuplicateUsername)));
        }

        #[tokio::test]
        async fn correctly_creates_new_user_from_request() {
            let alice_reg = alice_registration();
            let alice_reg_clone = alice_reg.clone();
            let alice_u = alice_user();
            let alice_u_clone = alice_u.clone();
            let token = "bwAB-924+2";

            let mut mock_provider = MockAuthProvider::new();
            mock_provider
                .expect_hash_pw()
                .with(eq(alice_reg_clone.password))
                .once()
                .return_once(|_| Ok(alice_u_clone.password_hash));
            mock_provider
                .expect_create_token()
                .with(eq(alice_u.id))
                .once()
                .return_once(|_| Ok(token.to_string()));

            let mock_repo = MockUserRepo {
                insert_new: Some(Box::new(move |u| {
                    assert!(
                        u.name == alice_reg_clone.name
                            && u.email == alice_reg_clone.email
                            && u.username == alice_reg_clone.username
                            && u.password_hash == alice_u.password_hash
                    );
                    Ok(alice_u.clone())
                })),
                ..Default::default()
            };

            let auth = AuthenticatorSvc::new(fake_pool(), mock_repo, mock_provider);
            let result = auth.signup(alice_reg).await;

            assert!(matches!(result, Ok(t) if t == token));
        }
    }

    mod login {
        use super::*;

        #[tokio::test]
        async fn handles_missing_account() {
            let (email, pw) = ("man@plan.ca", "#caMan-pl4n");

            let mock_repo = MockUserRepo {
                get_by_email: Some(Box::new(move |e| {
                    assert_eq!(email, e);
                    Ok(None)
                })),
                ..Default::default()
            };

            let auth = AuthenticatorSvc::new(fake_pool(), mock_repo, MockAuthProvider::new());
            assert!(matches!(
                auth.login(email, pw).await,
                Err(AuthError::NotFound)
            ));
        }

        #[tokio::test]
        async fn handles_incorrect_password() {
            let alice = alice_user();
            let alice_clone = alice.clone();
            let invalid_pw = "this will be mocked";

            let mock_repo = MockUserRepo {
                get_by_email: Some(Box::new(move |e| {
                    assert_eq!(alice_clone.email, e);
                    Ok(Some(alice_clone.clone()))
                })),
                ..Default::default()
            };

            let mut mock_provider = MockAuthProvider::new();
            mock_provider
                .expect_is_valid_pw()
                .with(eq(invalid_pw), eq(alice.password_hash))
                .once()
                .return_once(|_, _| Ok(false));

            let auth = AuthenticatorSvc::new(fake_pool(), mock_repo, mock_provider);
            assert!(matches!(
                auth.login(&alice.email, invalid_pw).await,
                Err(AuthError::InvalidPassword)
            ));
        }

        #[tokio::test]
        async fn creates_token_for_valid_credentials() {
            let alice = alice_user();
            let alice_clone = alice.clone();
            let correct_pw = "this will be mocked";
            let token = "123_token_yeah";

            let mock_repo = MockUserRepo {
                get_by_email: Some(Box::new(move |e| {
                    assert_eq!(alice_clone.email, e);
                    Ok(Some(alice_clone.clone()))
                })),
                ..Default::default()
            };

            let mut mock_provider = MockAuthProvider::new();
            mock_provider
                .expect_is_valid_pw()
                .with(eq(correct_pw), eq(alice.password_hash))
                .once()
                .return_once(|_, _| Ok(true));
            mock_provider
                .expect_create_token()
                .with(eq(alice.id))
                .once()
                .return_once(|_| Ok(token.to_string()));

            let auth = AuthenticatorSvc::new(fake_pool(), mock_repo, mock_provider);
            assert!(matches!(
                auth.login(&alice.email, correct_pw).await,
                Ok(t) if t == token
            ));
        }
    }

    // Determined that testing `validate_token` would be trivial
}
