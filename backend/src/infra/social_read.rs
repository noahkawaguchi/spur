use crate::read::{ReadError, SocialRead};
use sqlx::PgPool;

pub struct PgSocialRead {
    pool: PgPool,
}

impl PgSocialRead {
    pub const fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait::async_trait]
impl SocialRead for PgSocialRead {
    async fn friend_usernames(&self, id: i32) -> Result<Vec<String>, ReadError> {
        sqlx::query_scalar!(
            "
            SELECT username FROM users WHERE id IN (
                SELECT
                    CASE
                        WHEN lesser_id = $1 THEN greater_id
                        ELSE lesser_id
                    END AS friend_id
                FROM friendship
                WHERE confirmed_at IS NOT NULL
                AND (lesser_id = $1 OR greater_id = $1)
            )
            ",
            id,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn pending_requests(&self, id: i32) -> Result<Vec<String>, ReadError> {
        sqlx::query_scalar!(
            "
            SELECT username FROM users WHERE id IN (
                SELECT lesser_id AS requester_id FROM friendship
                WHERE confirmed_at IS NULL
                AND greater_id = $1
                AND lesser_requested

                UNION ALL

                SELECT greater_id AS requester_id FROM friendship
                WHERE confirmed_at IS NULL
                AND lesser_id = $1
                AND NOT lesser_requested
            )
            ",
            id,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::friendship::{repository::FriendshipStore, user_id_pair::UserIdPair},
        repository::friendship::FriendshipRepo,
        test_utils::{seed_data::seed_users, temp_db::with_test_pool},
    };

    #[tokio::test]
    async fn gets_all_requests_and_friends() {
        with_test_pool(|pool| async move {
            let read = PgSocialRead::new(pool.clone());
            let repo = FriendshipRepo;
            let [u1, u2, _, _] = seed_users(pool.clone()).await;

            let ids1 = UserIdPair::new(1, 3).unwrap();
            let ids2 = UserIdPair::new(2, 3).unwrap();

            // No requests, no friends
            let requests = read
                .pending_requests(3)
                .await
                .expect("failed to get empty requests");
            assert!(requests.is_empty());
            let friends = read
                .friend_usernames(3)
                .await
                .expect("failed to get empty friends");
            assert!(friends.is_empty());

            // Two requests, no friends
            repo.new_request(&pool, &ids1, 1)
                .await
                .expect("failed to create new request");
            repo.new_request(&pool, &ids2, 2)
                .await
                .expect("failed to create new request");
            let requests = read
                .pending_requests(3)
                .await
                .expect("failed to get requests");
            assert_eq!(requests, vec![u1.username.clone(), u2.username.clone()]);
            let friends = read
                .friend_usernames(3)
                .await
                .expect("failed to get empty friends");
            assert!(friends.is_empty());

            // One request, one friend
            repo.accept_request(&pool, &ids1)
                .await
                .expect("failed to accept request");
            let requests = read
                .pending_requests(3)
                .await
                .expect("failed to get single request");
            assert_eq!(requests, vec![u2.username.clone()]);
            let friends = read
                .friend_usernames(3)
                .await
                .expect("failed to get single friend");
            assert_eq!(friends, vec![u1.username.clone()]);

            // No requests, two friends
            repo.accept_request(&pool, &ids2)
                .await
                .expect("failed to accept request");
            let requests = read
                .pending_requests(3)
                .await
                .expect("failed to get empty requests");
            assert!(requests.is_empty());
            let friends = read
                .friend_usernames(3)
                .await
                .expect("failed to get friends");
            assert_eq!(friends, vec![u1.username, u2.username]);
        })
        .await;
    }
}
