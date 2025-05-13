use crate::models::{NewUser, User};
use sqlx::PgPool;

pub async fn insert_new(pool: &PgPool, new_user: &NewUser<'_>) -> sqlx::Result<()> {
    let _ = sqlx::query!(
        "INSERT INTO users (name, email, username, password_hash) VALUES ($1, $2, $3, $4)",
        new_user.name,
        new_user.email,
        new_user.username,
        new_user.password_hash,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_by_email(pool: &PgPool, email: &str) -> sqlx::Result<User> {
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE email = $1", email)
        .fetch_one(pool)
        .await?;

    Ok(user)
}

pub async fn get_by_username(pool: &PgPool, username: &str) -> sqlx::Result<User> {
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE username = $1", username)
        .fetch_one(pool)
        .await?;

    Ok(user)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::with_test_pool;
    use chrono::Utc;

    fn make_test_users<'a>() -> Vec<NewUser<'a>> {
        vec![
            NewUser {
                name: "Alice",
                email: "alice@example.com",
                username: "alice123",
                password_hash: "Ga45392*&$asd$",
            },
            NewUser {
                name: "Bob",
                email: "bob@email.com",
                username: "bob456",
                password_hash: "$$%%wub2",
            },
            NewUser {
                name: "Carla",
                email: "carla@mail.org",
                username: "carla789",
                password_hash: "95@fa@fF%aaa",
            },
        ]
    }

    #[tokio::test]
    async fn inserts_and_gets_users() {
        with_test_pool(|pool| async move {
            let test_users = make_test_users();

            // Insert
            for user in &test_users {
                insert_new(&pool, user)
                    .await
                    .expect("failed to insert test user");
            }

            // Get by email
            for user in &test_users {
                let got_by_email = get_by_email(&pool, user.email)
                    .await
                    .expect("failed to get user by email");

                assert_eq!(got_by_email, user);
            }

            // Get by username
            for user in &test_users {
                let got_by_username = get_by_username(&pool, user.username)
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
            for (i, user) in make_test_users().into_iter().enumerate() {
                insert_new(&pool, &user)
                    .await
                    .expect("failed to insert user");
                let created_time = Utc::now();

                let got_user = get_by_email(&pool, user.email)
                    .await
                    .expect("failed to get user");

                // created_at should be within one second of the approximate time created
                assert!((got_user.created_at - created_time).num_seconds().abs() <= 1);

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
            let real_alice = NewUser {
                name: "Alice",
                email: "alice@example.com",
                username: "alice123",
                password_hash: "Ga45392*&$asd$",
            };

            let fake_alice = NewUser {
                name: "Alice Again",
                email: "alice@example.com",
                username: "alice999",
                password_hash: "Ga45392*&$asd$",
            };

            insert_new(&pool, &real_alice)
                .await
                .expect("failed to insert real Alice");

            let result = insert_new(&pool, &fake_alice).await;

            assert!(matches!(result, Err(sqlx::Error::Database(_))));
        })
        .await;
    }

    #[tokio::test]
    async fn rejects_duplicate_username() {
        with_test_pool(|pool| async move {
            let real_bob = NewUser {
                name: "Bob",
                email: "bob@email.com",
                username: "bob456",
                password_hash: "$$%%wub2",
            };

            let fake_bob = NewUser {
                name: "I'm Bob",
                email: "bob_bob@bob.bob",
                username: "bob456",
                password_hash: "$$%%wub2",
            };

            insert_new(&pool, &real_bob)
                .await
                .expect("failed to insert real Bob");

            let result = insert_new(&pool, &fake_bob).await;

            assert!(matches!(result, Err(sqlx::Error::Database(_))));
        })
        .await;
    }

    #[tokio::test]
    async fn rejects_empty_and_blank_fields() {
        with_test_pool(|pool| async move {
            let complete_user = NewUser {
                name: "Carla",
                email: "carla@mail.org",
                username: "carla789",
                password_hash: "95@fa@fF%aaa",
            };

            let incomplete_users = vec![
                NewUser { name: "", ..complete_user },
                NewUser { name: "  ", ..complete_user },
                NewUser { email: "", ..complete_user },
                NewUser { email: "   ", ..complete_user },
                NewUser { username: "", ..complete_user },
                NewUser { username: " ", ..complete_user },
                NewUser { password_hash: "", ..complete_user },
                NewUser { password_hash: "      ", ..complete_user },
            ];

            for user in incomplete_users {
                let result = insert_new(&pool, &user).await;
                assert!(matches!(result, Err(sqlx::Error::Database(_))));
            }
        })
        .await;
    }
}
