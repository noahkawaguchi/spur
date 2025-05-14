use crate::{
    models::{NewUser, User},
    services::auth_svc::UserRepository,
};

#[derive(Clone)]
pub struct UserRepo {
    pool: sqlx::PgPool,
}

impl UserRepo {
    pub const fn new(pool: sqlx::PgPool) -> Self { Self { pool } }
}

#[async_trait::async_trait]
impl UserRepository for UserRepo {
    async fn insert_new(&self, new_user: &NewUser) -> sqlx::Result<()> {
        let _ = sqlx::query!(
            "INSERT INTO users (name, email, username, password_hash) VALUES ($1, $2, $3, $4)",
            new_user.name,
            new_user.email,
            new_user.username,
            new_user.password_hash,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_by_email(&self, email: &str) -> sqlx::Result<User> {
        let user = sqlx::query_as!(User, "SELECT * FROM users WHERE email = $1", email)
            .fetch_one(&self.pool)
            .await?;

        Ok(user)
    }

    async fn get_by_username(&self, username: &str) -> sqlx::Result<User> {
        let user = sqlx::query_as!(User, "SELECT * FROM users WHERE username = $1", username)
            .fetch_one(&self.pool)
            .await?;

        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{with_test_pool, within_one_second};
    use chrono::Utc;

    fn make_test_users() -> Vec<NewUser> {
        vec![
            NewUser {
                name: String::from("Alice"),
                email: String::from("alice@example.com"),
                username: String::from("alice123"),
                password_hash: String::from("Ga45392*&$asd$"),
            },
            NewUser {
                name: String::from("Bob"),
                email: String::from("bob@email.com"),
                username: String::from("bob456"),
                password_hash: String::from("$$%%wub2"),
            },
            NewUser {
                name: String::from("Carla"),
                email: String::from("carla@mail.org"),
                username: String::from("carla789"),
                password_hash: String::from("95@fa@fF%aaa"),
            },
        ]
    }

    #[tokio::test]
    async fn inserts_and_gets_users() {
        with_test_pool(|pool| async move {
            let test_users = make_test_users();
            let repo = UserRepo::new(pool);

            // Insert
            for user in &test_users {
                repo.insert_new(user)
                    .await
                    .expect("failed to insert test user");
            }

            // Get by email
            for user in &test_users {
                let got_by_email = repo
                    .get_by_email(&user.email)
                    .await
                    .expect("failed to get user by email");

                assert_eq!(got_by_email, user);
            }

            // Get by username
            for user in &test_users {
                let got_by_username = repo
                    .get_by_username(&user.username)
                    .await
                    .expect("failed to get user by username");

                assert_eq!(got_by_username, user);
            }
        })
        .await;
    }

    #[tokio::test]
    async fn sets_auto_generated_id_and_created_at() {
        with_test_pool(|pool| async move {
            let repo = UserRepo::new(pool);

            for (i, user) in make_test_users().into_iter().enumerate() {
                repo.insert_new(&user).await.expect("failed to insert user");
                let created_time = Utc::now();

                let got_user = repo
                    .get_by_email(&user.email)
                    .await
                    .expect("failed to get user");

                // created_at should be within one second of the approximate time created
                assert!(within_one_second(got_user.created_at, created_time));

                // id should increment starting from 1
                let expected_id: i32 = (i + 1).try_into().expect("failed to cast usize into i32");
                assert_eq!(got_user.id, expected_id);
            }
        })
        .await;
    }

    #[tokio::test]
    async fn rejects_duplicate_email() {
        with_test_pool(|pool| async move {
            let repo = UserRepo::new(pool);

            let real_alice = NewUser {
                name: String::from("Alice"),
                email: String::from("alice@example.com"),
                username: String::from("alice123"),
                password_hash: String::from("Ga45392*&$asd$"),
            };

            let fake_alice = NewUser {
                name: String::from("Alice Again"),
                email: String::from("alice@example.com"),
                username: String::from("alice999"),
                password_hash: String::from("Ga45392*&$asd$"),
            };

            repo.insert_new(&real_alice)
                .await
                .expect("failed to insert real Alice");

            let result = repo.insert_new(&fake_alice).await;

            assert!(matches!(result, Err(sqlx::Error::Database(_))));
        })
        .await;
    }

    #[tokio::test]
    async fn rejects_duplicate_username() {
        with_test_pool(|pool| async move {
            let repo = UserRepo::new(pool);

            let real_bob = NewUser {
                name: String::from("Bob"),
                email: String::from("bob@email.com"),
                username: String::from("bob456"),
                password_hash: String::from("$$%%wub2"),
            };

            let fake_bob = NewUser {
                name: String::from("I'm Bob"),
                email: String::from("bob_bob@bob.bob"),
                username: String::from("bob456"),
                password_hash: String::from("$$%%wub2"),
            };

            repo.insert_new(&real_bob)
                .await
                .expect("failed to insert real Bob");

            let result = repo.insert_new(&fake_bob).await;

            assert!(matches!(result, Err(sqlx::Error::Database(_))));
        })
        .await;
    }

    #[tokio::test]
    async fn rejects_empty_and_blank_fields() {
        with_test_pool(|pool| async move {
            let repo = UserRepo::new(pool);

            let complete_user = NewUser {
                name: String::from("Carla"),
                email: String::from("carla@mail.org"),
                username: String::from("carla789"),
                password_hash: String::from("95@fa@fF%aaa"),
            };

            let incomplete_users = vec![
                NewUser { name: String::new(), ..complete_user.clone() },
                NewUser { name: String::from("  "), ..complete_user.clone() },
                NewUser { email: String::new(), ..complete_user.clone() },
                NewUser { email: String::from("   "), ..complete_user.clone() },
                NewUser { username: String::new(), ..complete_user.clone() },
                NewUser { username: String::from(" "), ..complete_user.clone() },
                NewUser { password_hash: String::new(), ..complete_user.clone() },
                NewUser { password_hash: String::from("      "), ..complete_user },
            ];

            for user in incomplete_users {
                let result = repo.insert_new(&user).await;
                assert!(matches!(result, Err(sqlx::Error::Database(_))));
            }
        })
        .await;
    }
}
