use crate::{
    models::post::PostWithAuthor,
    read_models::{ReadError, SocialRead},
};
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
            SELECT u.username
            FROM users u
            JOIN (
                SELECT
                    confirmed_at,
                    CASE
                        WHEN lesser_id = $1 THEN greater_id
                        ELSE lesser_id
                    END AS friend_id
                FROM friendship
                WHERE confirmed_at IS NOT NULL
                    AND (lesser_id = $1 OR greater_id = $1)
            ) AS f ON f.friend_id = u.id
            ORDER BY f.confirmed_at DESC
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
            SELECT u.username
            FROM users u
            JOIN (
                SELECT lesser_id AS requester_id, requested_at
                FROM friendship
                WHERE confirmed_at IS NULL
                    AND greater_id = $1
                    AND lesser_requested

                UNION ALL

                SELECT greater_id AS requester_id, requested_at
                FROM friendship
                WHERE confirmed_at IS NULL
                    AND lesser_id = $1
                    AND NOT lesser_requested
            ) AS f ON f.requester_id = u.id
            ORDER BY f.requested_at DESC
            ",
            id,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn friend_posts(&self, user_id: i32) -> Result<Vec<PostWithAuthor>, ReadError> {
        sqlx::query_as!(
            PostWithAuthor,
            "
            SELECT p.*, u.username AS author_username
            FROM post p
            LEFT JOIN users u ON p.author_id = u.id
            JOIN (
                SELECT
                    CASE
                        WHEN f.lesser_id = $1 THEN f.greater_id
                        ELSE f.lesser_id
                    END AS friend_id
                FROM friendship f
                WHERE f.confirmed_at IS NOT NULL AND (f.lesser_id = $1 OR f.greater_id = $1)
            ) AS friends
            ON p.author_id = friends.friend_id
            ORDER BY p.created_at DESC
            ",
            user_id,
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
        domain::{
            friendship::{FriendshipRepo, user_id_pair::UserIdPair},
            post::PostRepo,
        },
        infra::{
            friendship_repo::PgFriendshipRepo, post_repo::PgPostRepo,
            post_with_author_read::PgPostWithAuthorRead,
        },
        read_models::PostWithAuthorRead,
        test_utils::{
            seed_data::{seed_friends, seed_root_post, seed_users},
            temp_db::with_test_pool,
        },
    };

    #[tokio::test]
    async fn gets_all_requests_and_friends() {
        with_test_pool(|pool| async move {
            let read = PgSocialRead::new(pool.clone());
            let repo = PgFriendshipRepo;
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
            // Most recently requested should be first
            assert_eq!(requests, vec![u2.username.clone(), u1.username.clone()]);
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
            // Most recently accepted should be first
            assert_eq!(friends, vec![u2.username, u1.username]);
        })
        .await;
    }

    #[tokio::test]
    async fn gets_only_posts_by_friends_of_a_user() {
        with_test_pool(|pool| async move {
            seed_users(pool.clone()).await;
            seed_root_post(&pool).await;
            seed_friends(pool.clone()).await;

            let read = PgSocialRead::new(pool.clone());
            let post_with_author_read = PgPostWithAuthorRead::new(pool.clone());
            let repo = PgPostRepo;

            let u1p2_body = "User one post two";
            let u1p3_body = "User one post three";
            let u2p1_body = "User two post one";
            let u2p2_body = "User two post two";
            let u3p1_body = "User three post one";
            let u3p2_body = "User three post two";
            let u4p1_body = "User four post one";
            let u4p2_body = "User four post two";

            repo.insert_new(&pool, 4, 1, u4p1_body).await.unwrap(); // ID 2
            repo.insert_new(&pool, 3, 1, u3p1_body).await.unwrap(); // ID 3
            repo.insert_new(&pool, 2, 1, u2p1_body).await.unwrap(); // ID 4
            repo.insert_new(&pool, 1, 2, u1p2_body).await.unwrap(); // ID 5
            repo.insert_new(&pool, 4, 3, u4p2_body).await.unwrap(); // ID 6
            repo.insert_new(&pool, 3, 2, u3p2_body).await.unwrap(); // ID 7
            repo.insert_new(&pool, 2, 2, u2p2_body).await.unwrap(); // ID 8
            repo.insert_new(&pool, 1, 3, u1p3_body).await.unwrap(); // ID 9

            let u2p1 = post_with_author_read.by_post_id(4).await.unwrap();
            let u2p2 = post_with_author_read.by_post_id(8).await.unwrap();
            let u3p1 = post_with_author_read.by_post_id(3).await.unwrap();
            let u3p2 = post_with_author_read.by_post_id(7).await.unwrap();
            let u4p1 = post_with_author_read.by_post_id(2).await.unwrap();
            let u4p2 = post_with_author_read.by_post_id(6).await.unwrap();

            let u1_friend_posts = read.friend_posts(1).await.unwrap();
            let u2_friend_posts = read.friend_posts(2).await.unwrap();
            let u3_friend_posts = read.friend_posts(3).await.unwrap();
            let u4_friend_posts = read.friend_posts(4).await.unwrap();

            // 1 has no friends
            assert!(u1_friend_posts.is_empty());
            // 2 is friends with both 3 and 4
            assert_eq!(u2_friend_posts, vec![u3p2, u4p2, u3p1, u4p1,]);
            // 3 and 4 are each only friends with 2
            let u2_posts = vec![u2p2, u2p1];
            assert_eq!(u3_friend_posts, u2_posts);
            assert_eq!(u4_friend_posts, u2_posts);
        })
        .await;
    }
}
