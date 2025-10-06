use anyhow::Result;
use sqlx::PgPool;
use std::env;

struct SeedUser {
    name: &'static str,
    email: &'static str,
    username: &'static str,
    password_hash: String,
}

pub async fn seed(pool: &PgPool) -> Result<()> {
    let users = [
        SeedUser {
            name: "Alice Example",
            email: "alice@example.com",
            username: "alice123",
            password_hash: hash_env("ALICE_PW")?,
        },
        SeedUser {
            name: "Bob Bobby",
            email: "bob@email.site",
            username: "R0B3RT",
            password_hash: hash_env("BOB_PW")?,
        },
        SeedUser {
            name: "Charlene Lean",
            email: "char@lean.me",
            username: "c-h-a-r",
            password_hash: hash_env("CHAR_PW")?,
        },
        SeedUser {
            name: "Diego Dickens",
            email: "diego@dickens.org",
            username: "_go654b2_",
            password_hash: hash_env("DIEGO_PW")?,
        },
        SeedUser {
            name: "Emi Mimi",
            email: "mimimi@me.jp",
            username: "M-_-E",
            password_hash: hash_env("EMI_PW")?,
        },
    ];

    for user in users {
        sqlx::query!(
            "
            INSERT INTO users (name, email, username, password_hash)
            VALUES ($1::text, $2::text, $3::text, $4::text)
            ",
            user.name,
            user.email,
            user.username,
            user.password_hash,
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}

fn hash_env(key: &'static str) -> Result<String> {
    bcrypt::hash(env::var(key)?, bcrypt::DEFAULT_COST).map_err(Into::into)
}
