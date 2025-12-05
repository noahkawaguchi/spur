use crate::domain::{
    RepoError,
    friendship::{FriendshipRepo, FriendshipStatus, user_id_pair::UserIdPair},
};
use sqlx::PgExecutor;

pub struct PgFriendshipRepo;

#[async_trait::async_trait]
impl FriendshipRepo for PgFriendshipRepo {
    async fn new_request(
        &self,
        exec: impl PgExecutor<'_>,
        ids: &UserIdPair,
        requester_id: i32,
    ) -> Result<(), RepoError> {
        sqlx::query!(
            "
            INSERT INTO friendship (lesser_id, greater_id, lesser_requested)
            VALUES ($1, $2, $3)
            ",
            ids.lesser(),
            ids.greater(),
            ids.is_lesser(requester_id)?,
        )
        .execute(exec)
        .await
        .map_err(Into::into)
        .map(|_| ())
    }

    async fn accept_request(
        &self,
        exec: impl PgExecutor<'_>,
        ids: &UserIdPair,
    ) -> Result<(), RepoError> {
        sqlx::query!(
            "
            UPDATE friendship
            SET confirmed_at = CURRENT_TIMESTAMP
            WHERE lesser_id = $1 AND greater_id = $2
            ",
            ids.lesser(),
            ids.greater(),
        )
        .execute(exec)
        .await
        .map_err(Into::into)
        .map(|_| ())
    }

    async fn get_status(
        &self,
        exec: impl PgExecutor<'_>,
        ids: &UserIdPair,
    ) -> Result<FriendshipStatus, RepoError> {
        sqlx::query!(
            r#"
            SELECT lesser_requested, confirmed_at IS NOT NULL AS "confirmed!"
            FROM friendship
            WHERE lesser_id = $1 AND greater_id = $2
            "#,
            ids.lesser(),
            ids.greater(),
        )
        .fetch_optional(exec)
        .await
        .map_err(Into::into)
        .map(|row| match row {
            None => FriendshipStatus::Nil,
            Some(friends) if friends.confirmed => FriendshipStatus::Friends,
            Some(pair) if pair.lesser_requested => FriendshipStatus::PendingFrom(ids.lesser()),
            Some(_) => FriendshipStatus::PendingFrom(ids.greater()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{
        seed_data::seed_users, temp_db::with_test_pool, time::within_five_seconds, tokio_test,
    };
    use chrono::{DateTime, Utc};
    use sqlx::PgPool;

    struct Friendship {
        lesser_id: i32,
        greater_id: i32,
        lesser_requested: bool,
        requested_at: DateTime<Utc>,
        confirmed_at: Option<DateTime<Utc>>,
    }

    async fn must_get_friendship(pool: &PgPool, first_id: i32, second_id: i32) -> Friendship {
        sqlx::query_as!(
            Friendship,
            "SELECT * FROM friendship WHERE lesser_id = $1 AND greater_id = $2",
            first_id,
            second_id,
        )
        .fetch_one(pool)
        .await
        .expect("failed to get friendship")
    }

    #[test]
    fn sets_initial_values_on_insertion() {
        tokio_test(async {
            with_test_pool(|pool| async move {
                let repo = PgFriendshipRepo;
                seed_users(&pool).await;

                repo.new_request(&pool, &UserIdPair::new(1, 2).unwrap(), 1)
                    .await
                    .expect("failed to insert new request");

                let friendship = must_get_friendship(&pool, 1, 2).await;

                assert_eq!(friendship.lesser_id, 1);
                assert_eq!(friendship.greater_id, 2);
                assert!(friendship.lesser_requested);
                assert!(within_five_seconds(friendship.requested_at, Utc::now()));
                assert!(friendship.confirmed_at.is_none());
            })
            .await;
        });
    }

    #[test]
    fn updates_values_for_accepted_request() {
        tokio_test(async {
            with_test_pool(|pool| async move {
                let repo = PgFriendshipRepo;
                seed_users(&pool).await;
                let ids = UserIdPair::new(1, 3).unwrap();

                repo.new_request(&pool, &ids, 3)
                    .await
                    .expect("failed to insert new request");

                repo.accept_request(&pool, &ids)
                    .await
                    .expect("failed to accept request");

                let friendship = must_get_friendship(&pool, 1, 3).await;

                assert_eq!(friendship.lesser_id, 1);
                assert_eq!(friendship.greater_id, 3);
                assert!(!friendship.lesser_requested);
                assert!(within_five_seconds(friendship.requested_at, Utc::now()));
                assert!(within_five_seconds(
                    friendship
                        .confirmed_at
                        .expect("unexpected None confirmation time"),
                    Utc::now(),
                ));
            })
            .await;
        });
    }

    #[test]
    fn gets_all_four_possible_statuses() {
        tokio_test(async {
            with_test_pool(|pool| async move {
                let repo = PgFriendshipRepo;
                seed_users(&pool).await;

                let ids1 = UserIdPair::new(1, 3).unwrap();
                let ids2 = UserIdPair::new(2, 3).unwrap();

                let status = repo
                    .get_status(&pool, &ids2)
                    .await
                    .expect("failed to get status");
                assert_eq!(status, FriendshipStatus::Nil);

                repo.new_request(&pool, &ids2, 3)
                    .await
                    .expect("failed to create new request");
                let status = repo
                    .get_status(&pool, &ids2)
                    .await
                    .expect("failed to get status");
                assert_eq!(status, FriendshipStatus::PendingFrom(3));

                repo.new_request(&pool, &ids1, 1)
                    .await
                    .expect("failed to create new request");
                let status = repo
                    .get_status(&pool, &ids1)
                    .await
                    .expect("failed to get status");
                assert_eq!(status, FriendshipStatus::PendingFrom(1));

                repo.accept_request(&pool, &ids2)
                    .await
                    .expect("failed to accept request");
                let status = repo
                    .get_status(&pool, &ids2)
                    .await
                    .expect("failed to get status");
                assert_eq!(status, FriendshipStatus::Friends);
            })
            .await;
        });
    }
}
