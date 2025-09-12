use crate::{
    domain::{
        friendship::{FriendshipRepo, user_id_pair::UserIdPair},
        user::UserRepo,
    },
    infra::{friendship_repo::PgFriendshipRepo, post_repo::PgPostRepo, user_repo::PgUserRepo},
    models::user::NewUser,
    test_utils::temp_db::with_test_pool,
};
use sqlx::PgPool;

/// Inserts four new users into the test database and returns them as they were inserted.
/// They will automatically be given IDs 1, 2, 3, and 4 if there are no other existing users.
///
/// # Panics
///
/// Panics if any of the insertions fail. This function should only be used in testing.
pub async fn seed_users(pool: sqlx::PgPool) -> [NewUser; 4] {
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
        .insert_new(&pool, &drake)
        .await
        .expect("failed to insert Drake");

    user_repo
        .insert_new(&pool, &eunice)
        .await
        .expect("failed to insert Eunice");

    user_repo
        .insert_new(&pool, &felipe)
        .await
        .expect("failed to insert Felipe");

    user_repo
        .insert_new(&pool, &gillian)
        .await
        .expect("failed to insert Gillian");

    [drake, eunice, felipe, gillian]
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
///
/// # Panics
///
/// Panics if any of the insertions fail. This function should only be used in testing.
pub async fn seed_friends(pool: sqlx::PgPool) {
    let two_and_three = UserIdPair::new(2, 3).unwrap();
    let two_and_four = UserIdPair::new(4, 2).unwrap();

    let repo = PgFriendshipRepo;

    // Confirmed requests
    repo.new_request(&pool, &two_and_three, 2).await.unwrap();
    repo.new_request(&pool, &two_and_four, 4).await.unwrap();
    repo.accept_request(&pool, &two_and_three).await.unwrap();
    repo.accept_request(&pool, &two_and_four).await.unwrap();

    // Unconfirmed requests
    repo.new_request(&pool, &UserIdPair::new(1, 3).unwrap(), 3)
        .await
        .unwrap();
    repo.new_request(&pool, &UserIdPair::new(4, 3).unwrap(), 3)
        .await
        .unwrap();
}

/// Inserts the "root" of the tree of posts, the only post allowed to have a NULL parent post. This
/// post will have an ID of 1. This is necessary for testing purposes, so that other posts can be
/// inserted in the normal fashion where a non-NULL parent post ID is required.
///
/// *Assumes a user with ID 1 already exists,* who will be the author of this post.
///
/// # Panics
///
/// Panics if the insertion fails for any reason.
pub async fn seed_root_post(pool: &sqlx::PgPool) {
    sqlx::query!("INSERT INTO post (author_id, parent_id, body) VALUES (1, NULL, 'root post')")
        .execute(pool)
        .await
        .expect("failed to insert root post");
}

/// Runs the provided test with a [`PostRepo`] instance that has users and the root post seeded.
pub async fn with_seeded_users_and_root_post<F, Fut>(test: F)
where
    F: FnOnce(PgPool, PgPostRepo, [NewUser; 4]) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    with_test_pool(|pool| async move {
        let new_users = seed_users(pool.clone()).await;
        seed_root_post(&pool).await;
        test(pool.clone(), PgPostRepo::new(pool), new_users).await;
    })
    .await;
}
