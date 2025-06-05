use crate::{
    domain::{
        content::repository::PromptStore,
        friendship::{repository::FriendshipStore, user_id_pair::UserIdPair},
        user::UserStore,
    },
    models::user::NewUser,
    repository::{friendship::FriendshipRepo, prompt::PromptRepo, user::UserRepo},
};
use spur_shared::models::PromptWithAuthor;

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

/// Inserts eight prompts into the test database, two by each of the four provided users, and
/// returns them in `PromptWithAuthor` form. The users are assumed to have IDs 1, 2, 3, and 4.
///
/// # Panics
///
/// Panics if any of the insertions fail. This function should only be used in testing.
pub async fn seed_prompts(pool: sqlx::PgPool, users: &[NewUser; 4]) -> [PromptWithAuthor; 8] {
    let [u1, u2, u3, u4] = users;

    let u1p1 = "User one prompt one";
    let u1p2 = "User one prompt two";
    let u2p1 = "User two prompt one";
    let u2p2 = "User two prompt two";
    let u3p1 = "User three prompt one";
    let u3p2 = "User three prompt two";
    let u4p1 = "User four prompt one";
    let u4p2 = "User four prompt two";

    let repo = PromptRepo::new(pool);

    let u1p1id = repo.insert_new(1, u1p1).await.unwrap();
    let u1p2id = repo.insert_new(1, u1p2).await.unwrap();
    let u2p1id = repo.insert_new(2, u2p1).await.unwrap();
    let u2p2id = repo.insert_new(2, u2p2).await.unwrap();
    let u3p1id = repo.insert_new(3, u3p1).await.unwrap();
    let u3p2id = repo.insert_new(3, u3p2).await.unwrap();
    let u4p1id = repo.insert_new(4, u4p1).await.unwrap();
    let u4p2id = repo.insert_new(4, u4p2).await.unwrap();

    let u1p1_with_author = PromptWithAuthor {
        id: u1p1id,
        author_username: u1.username.clone(),
        body: u1p1.to_string(),
    };

    let u1p2_with_author = PromptWithAuthor {
        id: u1p2id,
        author_username: u1.username.clone(),
        body: u1p2.to_string(),
    };

    let u2p1_with_author = PromptWithAuthor {
        id: u2p1id,
        author_username: u2.username.clone(),
        body: u2p1.to_string(),
    };

    let u2p2_with_author = PromptWithAuthor {
        id: u2p2id,
        author_username: u2.username.clone(),
        body: u2p2.to_string(),
    };

    let u3p1_with_author = PromptWithAuthor {
        id: u3p1id,
        author_username: u3.username.clone(),
        body: u3p1.to_string(),
    };

    let u3p2_with_author = PromptWithAuthor {
        id: u3p2id,
        author_username: u3.username.clone(),
        body: u3p2.to_string(),
    };

    let u4p1_with_author = PromptWithAuthor {
        id: u4p1id,
        author_username: u4.username.clone(),
        body: u4p1.to_string(),
    };

    let u4p2_with_author = PromptWithAuthor {
        id: u4p2id,
        author_username: u4.username.clone(),
        body: u4p2.to_string(),
    };

    [
        u1p1_with_author,
        u1p2_with_author,
        u2p1_with_author,
        u2p2_with_author,
        u3p1_with_author,
        u3p2_with_author,
        u4p1_with_author,
        u4p2_with_author,
    ]
}
