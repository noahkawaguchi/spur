use crate::{
    domain::friendship::{FriendshipStatus, repository::FriendshipStore, user_id_pair::UserIdPair},
    technical_error::TechnicalError,
};

pub struct FriendshipRepo {
    pool: sqlx::PgPool,
}

impl FriendshipRepo {
    pub const fn new(pool: sqlx::PgPool) -> Self { Self { pool } }
}

#[async_trait::async_trait]
impl FriendshipStore for FriendshipRepo {
    async fn new_request(&self, ids: &UserIdPair, requester_id: i32) -> Result<(), TechnicalError> {
        let _ = sqlx::query!(
            "
            INSERT INTO friendships (first_id, second_id, requester_first) 
            VALUES ($1, $2, $3)
            ",
            ids.lesser(),
            ids.greater(),
            ids.is_lesser(requester_id)?,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn accept_request(&self, ids: &UserIdPair) -> Result<(), TechnicalError> {
        let _ = sqlx::query!(
            "
            UPDATE friendships
            SET confirmed = TRUE, confirmed_at = CURRENT_TIMESTAMP
            WHERE first_id = $1 AND second_id = $2
            ",
            ids.lesser(),
            ids.greater(),
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_status(&self, ids: &UserIdPair) -> Result<FriendshipStatus, TechnicalError> {
        let row = sqlx::query!(
            "
            SELECT requester_first, confirmed FROM friendships
            WHERE first_id = $1 AND second_id = $2
            ",
            ids.lesser(),
            ids.greater(),
        )
        .fetch_optional(&self.pool)
        .await?;

        let status = match row {
            None => FriendshipStatus::Nil,
            Some(friends) if friends.confirmed => FriendshipStatus::Friends,
            Some(pair) if pair.requester_first => FriendshipStatus::PendingFrom(ids.lesser()),
            Some(_) => FriendshipStatus::PendingFrom(ids.greater()),
        };

        Ok(status)
    }

    async fn get_friends(&self, id: i32) -> Result<Vec<i32>, TechnicalError> {
        let friends = sqlx::query!(
            "
            SELECT
                CASE
                    WHEN first_id = $1 THEN second_id
                    ELSE first_id
                END AS friend_id
            FROM friendships
            WHERE confirmed
            AND (first_id = $1 OR second_id = $1)
            ",
            id,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(friends.into_iter().filter_map(|f| f.friend_id).collect())
    }

    async fn get_requests(&self, id: i32) -> Result<Vec<i32>, TechnicalError> {
        let requesters = sqlx::query!(
            "
            SELECT first_id AS requester_id FROM friendships
            WHERE NOT confirmed
            AND second_id = $1
            AND requester_first

            UNION ALL

            SELECT second_id AS requester_id FROM friendships
            WHERE NOT confirmed
            AND first_id = $1
            AND NOT requester_first
            ",
            id,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(requesters
            .into_iter()
            .filter_map(|r| r.requester_id)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::user::UserStore,
        models::user::NewUser,
        repository::user::UserRepo,
        test_utils::{with_test_pool, within_one_second},
    };
    use chrono::{DateTime, Utc};
    use sqlx::PgPool;

    struct Friendship {
        first_id: i32,
        second_id: i32,
        requester_first: bool,
        confirmed: bool,
        requested_at: DateTime<Utc>,
        confirmed_at: Option<DateTime<Utc>>,
    }

    async fn must_get_friendship(pool: PgPool, first_id: i32, second_id: i32) -> Friendship {
        sqlx::query_as!(
            Friendship,
            "SELECT * FROM friendships WHERE first_id = $1 AND second_id = $2",
            first_id,
            second_id,
        )
        .fetch_one(&pool)
        .await
        .expect("failed to get friendship")
    }

    async fn must_seed_users(pool: PgPool) {
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
    }

    #[tokio::test]
    async fn sets_initial_values_on_insertion() {
        with_test_pool(|pool| async move {
            let repo = FriendshipRepo::new(pool.clone());
            must_seed_users(pool.clone()).await;

            repo.new_request(&UserIdPair::new(1, 2).unwrap(), 1)
                .await
                .expect("failed to insert new request");

            let friendship = must_get_friendship(pool, 1, 2).await;

            assert_eq!(friendship.first_id, 1);
            assert_eq!(friendship.second_id, 2);
            assert!(friendship.requester_first);
            assert!(!friendship.confirmed);
            assert!(within_one_second(friendship.requested_at, Utc::now()));
            assert!(friendship.confirmed_at.is_none());
        })
        .await;
    }

    #[tokio::test]
    async fn updates_values_for_accepted_request() {
        with_test_pool(|pool| async move {
            let repo = FriendshipRepo::new(pool.clone());
            must_seed_users(pool.clone()).await;
            let ids = UserIdPair::new(1, 3).unwrap();

            repo.new_request(&ids, 3)
                .await
                .expect("failed to insert new request");

            repo.accept_request(&ids)
                .await
                .expect("failed to accept request");

            let friendship = must_get_friendship(pool, 1, 3).await;

            assert_eq!(friendship.first_id, 1);
            assert_eq!(friendship.second_id, 3);
            assert!(!friendship.requester_first);
            assert!(friendship.confirmed);
            assert!(within_one_second(friendship.requested_at, Utc::now()));
            assert!(within_one_second(
                friendship
                    .confirmed_at
                    .expect("unexpected None confirmation time"),
                Utc::now(),
            ));
        })
        .await;
    }

    /// Even though the enum has three variants, there are four possible statuses because a pending
    /// request can be from either the first user or the second user.
    #[tokio::test]
    async fn gets_all_four_possible_statuses() {
        with_test_pool(|pool| async move {
            let repo = FriendshipRepo::new(pool.clone());
            must_seed_users(pool).await;

            let ids1 = UserIdPair::new(1, 3).unwrap();
            let ids2 = UserIdPair::new(2, 3).unwrap();

            let status = repo.get_status(&ids2).await.expect("failed to get status");
            assert_eq!(status, FriendshipStatus::Nil);

            repo.new_request(&ids2, 3)
                .await
                .expect("failed to create new request");
            let status = repo.get_status(&ids2).await.expect("failed to get status");
            assert_eq!(status, FriendshipStatus::PendingFrom(3));

            repo.new_request(&ids1, 1)
                .await
                .expect("failed to create new request");
            let status = repo.get_status(&ids1).await.expect("failed to get status");
            assert_eq!(status, FriendshipStatus::PendingFrom(1));

            repo.accept_request(&ids2)
                .await
                .expect("failed to accept request");
            let status = repo.get_status(&ids2).await.expect("failed to get status");
            assert_eq!(status, FriendshipStatus::Friends);
        })
        .await;
    }

    #[tokio::test]
    async fn gets_all_requests_and_friends() {
        with_test_pool(|pool| async move {
            let repo = FriendshipRepo::new(pool.clone());
            must_seed_users(pool).await;

            let ids1 = UserIdPair::new(1, 3).unwrap();
            let ids2 = UserIdPair::new(2, 3).unwrap();

            // No requests, no friends
            let requests = repo
                .get_requests(3)
                .await
                .expect("failed to get empty requests");
            assert!(requests.is_empty());
            let friends = repo
                .get_friends(3)
                .await
                .expect("failed to get empty friends");
            assert!(friends.is_empty());

            // Two requests, no friends
            repo.new_request(&ids1, 1)
                .await
                .expect("failed to create new request");
            repo.new_request(&ids2, 2)
                .await
                .expect("failed to create new request");
            let requests = repo.get_requests(3).await.expect("failed to get requests");
            assert_eq!(requests, vec![1, 2]);
            let friends = repo
                .get_friends(3)
                .await
                .expect("failed to get empty friends");
            assert!(friends.is_empty());

            // One request, one friend
            repo.accept_request(&ids1)
                .await
                .expect("failed to accept request");
            let requests = repo
                .get_requests(3)
                .await
                .expect("failed to get single request");
            assert_eq!(requests, vec![2]);
            let friends = repo
                .get_friends(3)
                .await
                .expect("failed to get single friend");
            assert_eq!(friends, vec![1]);

            // No requests, two friends
            repo.accept_request(&ids2)
                .await
                .expect("failed to accept request");
            let requests = repo
                .get_requests(3)
                .await
                .expect("failed to get empty requests");
            assert!(requests.is_empty());
            let friends = repo.get_friends(3).await.expect("failed to get friends");
            assert_eq!(friends, vec![1, 2]);
        })
        .await;
    }
}
