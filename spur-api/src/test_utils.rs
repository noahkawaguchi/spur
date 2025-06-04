use crate::{domain::user::UserStore, models::user::NewUser, repository::user::UserRepo};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::{env, future};
use url::Url;
use uuid::Uuid;

/// Finds whether two `DateTime`s are within 1000ms of each other.
pub fn within_one_second(d1: DateTime<Utc>, d2: DateTime<Utc>) -> bool {
    (d1 - d2).num_milliseconds().abs() < 1000
}

/// Creates an ephemeral test database, runs migrations on it, runs the test, and then drops the
/// test database.
///
/// # Panics
///
/// Panics if any part of the database interaction process fails, providing a relevant panic
/// message. This function should only be used in testing.
pub async fn with_test_pool<F, Fut>(test: F)
where
    F: FnOnce(PgPool) -> Fut,
    Fut: future::Future<Output = ()>,
{
    // Establish the normal dev database connection in order to create the test database
    dotenvy::dotenv().expect("failed to load .env");
    let admin_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let admin_pool = PgPool::connect(&admin_url)
        .await
        .expect("failed to connect to admin pool");

    // Create a test database with a random name, alphanumeric only
    let test_db_name = format!("test{}", Uuid::new_v4().simple());
    sqlx::query(&format!(
        "CREATE DATABASE {}",
        assert_alphanumeric_identifier(&test_db_name),
    ))
    .execute(&admin_pool)
    .await
    .expect("failed to create test DB");

    // Connect to the test database
    let mut test_db_url = Url::parse(&admin_url).expect("failed to parse DATABASE_URL");
    test_db_url.set_path(&format!("/{test_db_name}"));
    let test_db_pool = PgPool::connect(test_db_url.as_str())
        .await
        .expect("failed to connect to test database");

    // Run all migrations in ./migrations on the test database
    sqlx::migrate!()
        .run(&test_db_pool)
        .await
        .expect("failed to run migrations");

    // Run the test
    test(test_db_pool.clone()).await;

    // Shut down the connection pool
    test_db_pool.close().await;

    // Drop the test database, terminating any remaining connections
    sqlx::query(&format!(
        "DROP DATABASE {} WITH (FORCE)",
        assert_alphanumeric_identifier(&test_db_name),
    ))
    .execute(&admin_pool)
    .await
    .expect("failed to drop test database");
}

fn assert_alphanumeric_identifier(s: &str) -> &str {
    let mut chars = s.chars();
    assert!(chars.next().is_some_and(|c| c.is_ascii_alphabetic()));
    assert!(chars.all(|c| c.is_ascii_alphanumeric()));
    s
}

/// Inserts four new users into the test database and returns them as they were inserted.
/// They will automatically be given IDs 1, 2, 3, and 4 if there are no other existing users.
///
/// # Panics
///
/// Panics if any of the insertions fail. This function should only be used in testing.
pub async fn must_seed_users(pool: PgPool) -> (NewUser, NewUser, NewUser, NewUser) {
    let user_repo = UserRepo::new(pool);

    let drake = NewUser {
        name: String::from("Drake"),
        email: String::from("drake@mail.cool"),
        username: String::from("drake_conan"),
        password_hash: String::from("ab45%2$#lLS"),
    };

    let eunice = NewUser {
        name: String::from("Eunice Lee"),
        email: String::from("eunice@lee.eee"),
        username: String::from("you_n_15"),
        password_hash: String::from("UNE$@$_b08088"),
    };

    let felipe = NewUser {
        name: String::from("Felipe Hall"),
        email: String::from("f.hall@mail-cloud.net"),
        username: String::from("fe_to_the_lip_to_the_e"),
        password_hash: String::from("ppaPpA44245$$$$"),
    };

    let gillian = NewUser {
        name: String::from("Gillian Lee"),
        email: String::from("gillian@lee.eee"),
        username: String::from("jill_e_ian_12345"),
        password_hash: String::from("these are not actually valid bcrypt hashes"),
    };

    user_repo
        .insert_new(&drake)
        .await
        .expect("failed to insert Drake");

    user_repo
        .insert_new(&eunice)
        .await
        .expect("failed to insert Eunice");

    user_repo
        .insert_new(&felipe)
        .await
        .expect("failed to insert Felipe");

    user_repo
        .insert_new(&gillian)
        .await
        .expect("failed to insert Gillian");

    (drake, eunice, felipe, gillian)
}
