use chrono::{DateTime, Utc};
use sqlx::{PgPool, query, query_as};

pub struct NewUser<'a> {
    pub name: &'a str,
    pub email: &'a str,
    pub username: &'a str,
    pub password_hash: &'a str,
}

pub struct User {
    id: i32,
    name: String,
    email: String,
    username: String,
    password_hash: String,
    created_at: DateTime<Utc>,
}

pub async fn insert_new(pool: &PgPool, new_user: &NewUser<'_>) -> sqlx::Result<()> {
    let _ = query!(
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
    let user = query_as!(User, "SELECT * FROM users WHERE email = $1", email)
        .fetch_one(pool)
        .await?;

    Ok(user)
}
