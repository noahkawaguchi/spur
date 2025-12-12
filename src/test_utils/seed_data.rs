use crate::{
    domain::{
        friendship::{FriendshipRepo, user_id_pair::UserIdPair},
        user::UserRepo,
    },
    infra::{friendship_repo::PgFriendshipRepo, user_repo::PgUserRepo},
    models::user::NewUser,
};
use anyhow::{Context, Result};
use sqlx::PgPool;

/// Inserts four new users into the test database and returns them as they were inserted.
/// They will automatically be given IDs 1, 2, 3, and 4 if there are no other existing users.
pub async fn seed_users(pool: &PgPool) -> Result<[NewUser; 4]> {
    let user_repo = PgUserRepo;

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
        .insert_new(pool, &drake)
        .await
        .context("failed to insert Drake")?;

    user_repo
        .insert_new(pool, &eunice)
        .await
        .context("failed to insert Eunice")?;

    user_repo
        .insert_new(pool, &felipe)
        .await
        .context("failed to insert Felipe")?;

    user_repo
        .insert_new(pool, &gillian)
        .await
        .context("failed to insert Gillian")?;

    Ok([drake, eunice, felipe, gillian])
}

/// Inserts friend requests and friendships into the test database, assuming users with IDs 1, 2,
/// 3, and 4 exist. Creates the following relationships:
///
/// ## By pair:
///
/// - Users 1 & 2 => no relation
/// - Users 1 & 3 => requested, unconfirmed
/// - Users 1 & 4 => no relation
/// - Users 2 & 3 => confirmed friends
/// - Users 2 & 4 => confirmed friends
/// - Users 3 & 4 => requested, unconfirmed
///
/// ## By individual (confirmed only):
///
/// - User 1 => no friends
/// - User 2 => friends with 3 and 4
/// - User 3 => friends with 2
/// - User 4 => friends with 2
pub async fn seed_friends(pool: &PgPool) -> Result<()> {
    let two_and_three = UserIdPair::new(2, 3)?;
    let two_and_four = UserIdPair::new(4, 2)?;

    let repo = PgFriendshipRepo;

    // Confirmed requests
    repo.new_request(pool, &two_and_three, 2).await?;
    repo.new_request(pool, &two_and_four, 4).await?;
    repo.accept_request(pool, &two_and_three).await?;
    repo.accept_request(pool, &two_and_four).await?;

    // Unconfirmed requests
    repo.new_request(pool, &UserIdPair::new(1, 3)?, 3).await?;
    repo.new_request(pool, &UserIdPair::new(4, 3)?, 3).await?;

    Ok(())
}

/// Inserts the "root" of the tree of posts, the only post allowed to have a NULL parent post. This
/// post will have an ID of 1. This is necessary for testing purposes, so that other posts can be
/// inserted in the normal fashion where a non-NULL parent post ID is required.
///
/// *Assumes a user with ID 1 already exists,* who will be the author of this post.
pub async fn seed_root_post(pool: &sqlx::PgPool) -> Result<()> {
    sqlx::query!("INSERT INTO post (author_id, parent_id, body) VALUES (1, NULL, 'root post')")
        .execute(pool)
        .await
        .context("failed to insert root post")
        .map(|_| ())
}

/// Seeds a test database with users (IDs 1, 2, 3, and 4) and the root post (by user 1).
pub async fn seed_users_and_root_post(pool: &PgPool) -> Result<[NewUser; 4]> {
    let new_users = seed_users(&pool).await?;
    seed_root_post(&pool).await?;
    Ok(new_users)
}
