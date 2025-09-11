use crate::{
    domain::user::{UserError, UserManager, UserRepo},
    models::user::{NewUser, User},
};

pub struct UserSvc<S: UserRepo> {
    store: S,
}

impl<S: UserRepo> UserSvc<S> {
    pub const fn new(store: S) -> Self { Self { store } }
}

#[async_trait::async_trait]
impl<S: UserRepo> UserManager for UserSvc<S> {
    async fn insert_new(&self, new_user: &NewUser) -> Result<User, UserError> {
        self.store.insert_new(new_user).await.map_err(Into::into)
    }

    async fn get_by_id(&self, id: i32) -> Result<User, UserError> {
        self.store.get_by_id(id).await?.ok_or(UserError::NotFound)
    }

    async fn get_by_email(&self, email: &str) -> Result<User, UserError> {
        self.store
            .get_by_email(email)
            .await?
            .ok_or(UserError::NotFound)
    }

    async fn get_by_username(&self, username: &str) -> Result<User, UserError> {
        self.store
            .get_by_username(username)
            .await?
            .ok_or(UserError::NotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{domain::user::MockUserRepo, repository::error::RepoError};
    use chrono::{Months, Utc};
    use mockall::predicate::eq;

    fn make_alice() -> User {
        User {
            id: 62,
            name: String::from("Alice Palace"),
            email: String::from("alice@palace.org"),
            username: String::from("al_is"),
            password_hash: String::from("ab%#S"),
            created_at: Utc::now(),
        }
    }

    fn make_bob() -> User {
        User {
            id: 9924,
            name: String::from("Roberta"),
            email: String::from("roberta@bob.eu"),
            username: String::from("bob_or_rob_or_ert"),
            password_hash: String::from("asd%$#iub8"),
            created_at: Utc::now(),
        }
    }

    fn make_charlie() -> User {
        User {
            id: 141_415,
            name: String::from("Charles McCharles"),
            email: String::from("charles@mc.com"),
            username: String::from("charlie_boy"),
            password_hash: String::from("ha$h3dp455w0rd"),
            created_at: Utc::now()
                .checked_sub_months(Months::new(12))
                .expect("failed to subtract 12 months from now"),
        }
    }

    mod insert_new {
        use super::*;

        fn new_user_alice() -> NewUser {
            NewUser {
                name: String::from("New Alice"),
                email: String::from("alice_new@example.com"),
                username: String::from("new_alice"),
                password_hash: String::from("$$hh%%hhYAY"),
            }
        }

        #[tokio::test]
        async fn errors_for_existing_email() {
            let alice = new_user_alice();
            let alice_clone = alice.clone();

            let mut mock_repo = MockUserRepo::new();
            mock_repo
                .expect_insert_new()
                .withf(move |u| {
                    u.name == alice_clone.name
                        && u.email == alice_clone.email
                        && u.username == alice_clone.username
                        && u.password_hash == alice_clone.password_hash
                })
                .once()
                .return_once(|_| {
                    Err(RepoError::UniqueViolation(String::from(
                        "users_email_unique",
                    )))
                });

            let user_svc = UserSvc::new(mock_repo);
            let result = user_svc.insert_new(&alice).await;

            assert!(matches!(result, Err(UserError::DuplicateEmail)));
        }

        #[tokio::test]
        async fn errors_for_existing_username() {
            let alice = new_user_alice();
            let alice_clone = alice.clone();

            let mut mock_repo = MockUserRepo::new();
            mock_repo
                .expect_insert_new()
                .withf(move |u| {
                    u.name == alice_clone.name
                        && u.email == alice_clone.email
                        && u.username == alice_clone.username
                        && u.password_hash == alice_clone.password_hash
                })
                .once()
                .return_once(|_| {
                    Err(RepoError::UniqueViolation(String::from(
                        "users_username_unique",
                    )))
                });

            let user_svc = UserSvc::new(mock_repo);
            let result = user_svc.insert_new(&alice).await;

            assert!(matches!(result, Err(UserError::DuplicateUsername)));
        }

        #[tokio::test]
        async fn correctly_creates_new_user_from_request() {
            let bob = make_bob();
            let bob_clone = bob.clone();
            let new_bob = NewUser::from(bob.clone());
            let new_bob_clone = new_bob.clone();

            let mut mock_repo = MockUserRepo::new();
            mock_repo
                .expect_insert_new()
                .withf(move |u| {
                    u.name == new_bob.name
                        && u.email == new_bob.email
                        && u.username == new_bob.username
                        && u.password_hash == new_bob.password_hash
                })
                .once()
                .return_once(move |_| Ok(bob));

            let user_svc = UserSvc::new(mock_repo);

            assert!(
                user_svc
                    .insert_new(&new_bob_clone)
                    .await
                    .is_ok_and(|returned_bob| returned_bob == bob_clone),
            );
        }
    }

    mod get_by {
        use super::*;

        #[tokio::test]
        async fn errors_for_nonexistent_users() {
            let nonexistent_id = 422;
            let nonexistent_email = "ghost@spectral.nz";
            let nonexistent_username = "not_real";

            let mut mock_user_repo = MockUserRepo::new();
            mock_user_repo
                .expect_get_by_id()
                .with(eq(nonexistent_id))
                .once()
                .return_once(|_| Ok(None));
            mock_user_repo
                .expect_get_by_email()
                .with(eq(nonexistent_email))
                .once()
                .return_once(|_| Ok(None));
            mock_user_repo
                .expect_get_by_username()
                .with(eq(nonexistent_username))
                .once()
                .return_once(|_| Ok(None));

            let user_svc = UserSvc::new(mock_user_repo);

            let id_result = user_svc.get_by_id(nonexistent_id).await;
            let email_result = user_svc.get_by_email(nonexistent_email).await;
            let username_result = user_svc.get_by_username(nonexistent_username).await;

            assert!(matches!(id_result, Err(UserError::NotFound)));
            assert!(matches!(email_result, Err(UserError::NotFound)));
            assert!(matches!(username_result, Err(UserError::NotFound)));
        }

        #[tokio::test]
        async fn retrieves_existing_users() {
            let alice = make_alice();
            let bob = make_bob();
            let charlie = make_charlie();

            let alice_clone = alice.clone();
            let bob_clone = bob.clone();
            let charlie_clone = charlie.clone();

            let bob_email = bob.email.clone();
            let charlie_username = charlie.username.clone();

            let mut mock_user_repo = MockUserRepo::new();
            mock_user_repo
                .expect_get_by_id()
                .with(eq(alice.id))
                .once()
                .return_once(|_| Ok(Some(alice_clone)));
            mock_user_repo
                .expect_get_by_email()
                .with(eq(bob_email))
                .once()
                .return_once(|_| Ok(Some(bob_clone)));
            mock_user_repo
                .expect_get_by_username()
                .with(eq(charlie_username))
                .once()
                .return_once(|_| Ok(Some(charlie_clone)));

            let user_svc = UserSvc::new(mock_user_repo);

            let id_result = user_svc
                .get_by_id(alice.id)
                .await
                .expect("failed to get by id");
            let email_result = user_svc
                .get_by_email(&bob.email)
                .await
                .expect("failed to get by email");
            let username_result = user_svc
                .get_by_username(&charlie.username)
                .await
                .expect("failed to get by username");

            assert_eq!(id_result, alice);
            assert_eq!(email_result, bob);
            assert_eq!(username_result, charlie);
        }
    }
}
