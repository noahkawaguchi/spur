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
