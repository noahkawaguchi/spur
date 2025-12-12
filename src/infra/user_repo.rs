use crate::{
    domain::{RepoError, user::UserRepo},
    models::user::{NewUser, User},
};
use sqlx::PgExecutor;

pub struct PgUserRepo;

#[async_trait::async_trait]
impl UserRepo for PgUserRepo {
    async fn insert_new(
        &self,
        exec: impl PgExecutor<'_>,
        new_user: &NewUser,
    ) -> Result<User, RepoError> {
        sqlx::query_as!(
            User,
            "
            INSERT INTO users (name, email, username, password_hash)
            VALUES ($1::text, $2::text, $3::text, $4::text)
            RETURNING *
            ",
            new_user.name,
            new_user.email,
            new_user.username,
            new_user.password_hash,
        )
        .fetch_one(exec)
        .await
        .map_err(Into::into)
    }

    async fn get_by_id(
        &self,
        exec: impl PgExecutor<'_>,
        id: i32,
    ) -> Result<Option<User>, RepoError> {
        sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
            .fetch_optional(exec)
            .await
            .map_err(Into::into)
    }

    async fn get_by_email(
        &self,
        exec: impl PgExecutor<'_>,
        email: &str,
    ) -> Result<Option<User>, RepoError> {
        sqlx::query_as!(User, "SELECT * FROM users WHERE email = $1", email)
            .fetch_optional(exec)
            .await
            .map_err(Into::into)
    }

    async fn get_by_username_exclusive(
        &self,
        exec: impl PgExecutor<'_>,
        username: &str,
    ) -> Result<Option<User>, RepoError> {
        sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE username = $1 FOR UPDATE",
            username
        )
        .fetch_optional(exec)
        .await
        .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{temp_db::with_test_pool, time::within_five_seconds, tokio_test};
    use anyhow::{Context, Result};
    use chrono::Utc;
    use sqlx::PgPool;

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

    #[sqlx::test]
    async fn inserts_and_gets_users(pool: PgPool) -> Result<()> {
        let test_users = make_test_users();
        let repo = PgUserRepo;

        // Insert
        for user in &test_users {
            repo.insert_new(&pool, user)
                .await
                .context("failed to insert test user")?;
        }

        // Get by ID (should automatically increment starting from 1)
        for (i, user) in test_users.iter().enumerate() {
            let got_by_id = repo
                .get_by_id(
                    &pool,
                    i32::try_from(i + 1).expect("failed to convert usize to i32"),
                )
                .await
                .context("failed to get user by ID")?
                .context("failed to get user by ID")?;

            assert_eq!(got_by_id, user);
        }

        // Get by email
        for user in &test_users {
            let got_by_email = repo
                .get_by_email(&pool, &user.email)
                .await
                .context("failed to get user by email")?
                .context("unexpected None user")?;

            assert_eq!(got_by_email, user);
        }

        // Get by username
        for user in &test_users {
            let got_by_username = repo
                .get_by_username_exclusive(&pool, &user.username)
                .await
                .context("failed to get user by username")?
                .context("unexpected None user")?;

            assert_eq!(got_by_username, user);
        }

        Ok(())
    }

    #[sqlx::test]
    async fn returns_none_for_nonexistent_user(pool: PgPool) -> Result<()> {
        let repo = PgUserRepo;

        let from_nonsense_email = repo.get_by_email(&pool, "nonsense@nothing.abc").await;
        let from_nonsense_username = repo
            .get_by_username_exclusive(&pool, "nonsensical_naan")
            .await;
        let from_nonsense_id = repo.get_by_id(&pool, 642).await;

        assert!(matches!(from_nonsense_email, Ok(None)));
        assert!(matches!(from_nonsense_username, Ok(None)));
        assert!(matches!(from_nonsense_id, Ok(None)));

        Ok(())
    }

    #[sqlx::test]
    async fn sets_auto_generated_id_and_created_at(pool: PgPool) -> Result<()> {
        let repo = PgUserRepo;

        for (i, user) in make_test_users().into_iter().enumerate() {
            let created_user = repo
                .insert_new(&pool, &user)
                .await
                .context("failed to insert user")?;

            assert!(within_five_seconds(created_user.created_at, Utc::now()));

            // id should increment starting from 1
            let expected_id: i32 = (i + 1)
                .try_into()
                .context("failed to cast usize into i32")?;

            assert_eq!(created_user.id, expected_id);
        }

        Ok(())
    }

    #[sqlx::test]
    async fn rejects_duplicate_email(pool: PgPool) -> Result<()> {
        let repo = PgUserRepo;

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

        repo.insert_new(&pool, &real_alice)
            .await
            .context("failed to insert real Alice")?;

        let result = repo.insert_new(&pool, &fake_alice).await;

        assert!(matches!(result, Err(RepoError::UniqueViolation(v)) if v == "users_email_unique"));

        Ok(())
    }

    #[sqlx::test]
    async fn rejects_duplicate_username(pool: PgPool) -> Result<()> {
        let repo = PgUserRepo;

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

        repo.insert_new(&pool, &real_bob)
            .await
            .context("failed to insert real Bob")?;

        let result = repo.insert_new(&pool, &fake_bob).await;

        assert!(
            matches!(result, Err(RepoError::UniqueViolation(v)) if v == "users_username_unique")
        );

        Ok(())
    }

    #[sqlx::test]
    async fn rejects_empty_and_blank_fields(pool: PgPool) -> Result<()> {
        let repo = PgUserRepo;

        let complete_user = NewUser {
            name: String::from("Carla"),
            email: String::from("carla@mail.org"),
            username: String::from("carla789"),
            password_hash: String::from("95@fa@fF%aaa"),
        };

        let incomplete_users = [
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
            assert!(matches!(
                repo.insert_new(&pool, &user).await,
                Err(RepoError::CheckViolation(v)) if v == "text_non_empty"
            ));
        }

        Ok(())
    }

    #[sqlx::test]
    async fn rejects_usernames_with_illegal_characters(pool: PgPool) -> Result<()> {
        let repo = PgUserRepo;

        let bad_usernames = [
            "$am",
            "dan123!",
            "sam the man",
            "sam　the　man",
            "sam\tthe_man",
            "dan\nthe_man",
            "sam\rthe_man",
            "サム・ザ・マン",
            "donny😂😂😂dan",
            "-–—ーdanielle〜~_",
        ];

        for username in bad_usernames {
            let sam = NewUser {
                name: String::from("Sam Dennis"),
                email: String::from("sam@dennis.de"),
                username: username.to_string(),
                password_hash: String::from("%$$aabbb1234"),
            };

            assert!(matches!(
                repo.insert_new(&pool, &sam).await,
                Err(RepoError::CheckViolation(v)) if v == "users_username_chars"
            ));
        }

        Ok(())
    }
}
