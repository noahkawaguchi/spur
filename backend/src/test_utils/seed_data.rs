use crate::{
    domain::{
        content::repository::PromptStore,
        friendship::{repository::FriendshipStore, user_id_pair::UserIdPair},
        user::UserStore,
    },
    models::{prompt::PromptWithAuthor, user::NewUser},
    repository::{friendship::FriendshipRepo, prompt::PromptRepo, user::UserRepo},
};

/// Inserts four new users into the test database and returns them as they were inserted.
/// They will automatically be given IDs 1, 2, 3, and 4 if there are no other existing users.
///
/// # Panics
///
/// Panics if any of the insertions fail. This function should only be used in testing.
pub async fn seed_users(pool: sqlx::PgPool) -> [NewUser; 4] {
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

    let repo = FriendshipRepo::new(pool.clone());

    // Confirmed requests
    repo.new_request(&two_and_three, 2).await.unwrap();
    repo.new_request(&two_and_four, 4).await.unwrap();
    repo.accept_request(&two_and_three).await.unwrap();
    repo.accept_request(&two_and_four).await.unwrap();

    // Unconfirmed requests
    repo.new_request(&UserIdPair::new(1, 3).unwrap(), 3)
        .await
        .unwrap();
    repo.new_request(&UserIdPair::new(4, 3).unwrap(), 3)
        .await
        .unwrap();
}

/// Inserts eight prompts into the test database, two by each of four users assumed to have IDs 1,
/// 2, 3, and 4, and returns the Prompts in `PromptWithAuthor` form.
///
/// # Panics
///
/// Panics if any of the insertions fail. This function should only be used in testing.
pub async fn seed_prompts(pool: sqlx::PgPool) -> [PromptWithAuthor; 8] {
    let repo = PromptRepo::new(pool);

    [
        repo.insert_new(1, "User one prompt one")
            .await
            .unwrap()
            .into(),
        repo.insert_new(1, "User one prompt two")
            .await
            .unwrap()
            .into(),
        repo.insert_new(2, "User two prompt one")
            .await
            .unwrap()
            .into(),
        repo.insert_new(2, "User two prompt two")
            .await
            .unwrap()
            .into(),
        repo.insert_new(3, "User three prompt one")
            .await
            .unwrap()
            .into(),
        repo.insert_new(3, "User three prompt two")
            .await
            .unwrap()
            .into(),
        repo.insert_new(4, "User four prompt one")
            .await
            .unwrap()
            .into(),
        repo.insert_new(4, "User four prompt two")
            .await
            .unwrap()
            .into(),
    ]
}
