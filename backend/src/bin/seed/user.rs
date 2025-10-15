use crate::time_utils::anchor_offset;
use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::env;

struct SeedUser {
    name: &'static str,
    email: &'static str,
    username: &'static str,
    password_hash: String,
    created_at: DateTime<Utc>,
}

/// Inserts six seed users into the database. The users will be given IDs 1 through 6 (if there are
/// no other users in the database, which there shouldn't be). User 1 is the "Spurt" character.
///
/// The following six environment variables must be set to the passwords to use for the dummy users:
/// `SPURT_PW`, `ALICE_PW`, `BOB_PW`, `CHAR_PW`, `DIEGO_PW`, `EMI_PW`.
pub async fn seed(pool: &PgPool) -> Result<()> {
    let users = [
        SeedUser {
            name: "Spurt",
            email: "no_support_email_yet@example.com",
            username: "spurt",
            password_hash: hash_env("SPURT_PW")?,
            created_at: anchor_offset(0, 0, 0)?,
        },
        SeedUser {
            name: "Alice Example",
            email: "alice@example.com",
            username: "alice123",
            password_hash: hash_env("ALICE_PW")?,
            created_at: anchor_offset(1, 1, 1)?,
        },
        SeedUser {
            name: "Bob Bobby",
            email: "bob@email.site",
            username: "R0B3RT",
            password_hash: hash_env("BOB_PW")?,
            created_at: anchor_offset(10, 12, 30)?,
        },
        SeedUser {
            name: "Charlene Lean",
            email: "char@lean.me",
            username: "c-h-a-r",
            password_hash: hash_env("CHAR_PW")?,
            created_at: anchor_offset(100, 8, 15)?,
        },
        SeedUser {
            name: "Diego Dickens",
            email: "diego@dickens.org",
            username: "_go654b2_",
            password_hash: hash_env("DIEGO_PW")?,
            created_at: anchor_offset(350, 23, 59)?,
        },
        SeedUser {
            name: "Emi Mimi",
            email: "mimimi@me.jp",
            username: "M-_-E",
            password_hash: hash_env("EMI_PW")?,
            created_at: anchor_offset(1000, 5, 5)?,
        },
    ];

    for user in users {
        sqlx::query!(
            "
            INSERT INTO users (name, email, username, password_hash, created_at)
            VALUES ($1::text, $2::text, $3::text, $4::text, $5)
            ",
            user.name,
            user.email,
            user.username,
            user.password_hash,
            user.created_at,
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}

fn hash_env(key: &'static str) -> Result<String> {
    bcrypt::hash(env::var(key)?, bcrypt::DEFAULT_COST).map_err(Into::into)
}
