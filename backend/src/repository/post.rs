use super::error::RepoError;
use crate::{
    domain::post::{PostInsertionOutcome, PostStore},
    models::post::PostInfo,
};
use anyhow::anyhow;

pub struct PostRepo {
    pool: sqlx::PgPool,
}

impl PostRepo {
    pub const fn new(pool: sqlx::PgPool) -> Self { Self { pool } }
}

#[async_trait::async_trait]
impl PostStore for PostRepo {
    async fn insert_new(
        &self,
        author_id: i32,
        parent_id: i32,
        body: &str,
    ) -> Result<PostInsertionOutcome, RepoError> {
        // Disallow writing posts in response to nonexistent, deleted, archived, or one's own posts
        sqlx::query_scalar!(
            "
            WITH parent AS (
                SELECT author_id, archived_at, deleted_at
                FROM post
                WHERE id = $2
                FOR UPDATE
            ),
            parent_status AS (
                SELECT
                    CASE
                        WHEN parent.deleted_at IS NOT NULL THEN 'deleted'
                        WHEN parent.archived_at IS NOT NULL THEN 'archived'
                        WHEN parent.author_id = $1 THEN 'self_reply'
                        ELSE 'ok'
                    END as status
                FROM parent
            ),
            _ AS (
                INSERT INTO post (author_id, parent_id, body)
                SELECT $1, $2, $3::text
                FROM parent_status
                WHERE status = 'ok'
            )
            SELECT status FROM parent_status
            ",
            author_id,
            parent_id,
            body,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Into::into) // Technical errors and unique violations
        .and_then(|status| {
            // Business rules enforced in SQL
            status
                .flatten()
                .map_or(Ok(PostInsertionOutcome::ParentMissing), |s| {
                    match s.as_str() {
                        "deleted" => Ok(PostInsertionOutcome::ParentDeleted),
                        "archived" => Ok(PostInsertionOutcome::ParentArchived),
                        "self_reply" => Ok(PostInsertionOutcome::SelfReply),
                        "ok" => Ok(PostInsertionOutcome::Inserted),
                        _ => Err(
                            anyhow!("Unexpected insertion status despite hardcoded strings").into(),
                        ),
                    }
                })
        })
    }

    async fn get_by_id(&self, id: i32) -> Result<Option<PostInfo>, RepoError> {
        sqlx::query_as!(
            PostInfo,
            "
            SELECT p.*, u.username AS author_username
            FROM post p
            LEFT JOIN users u ON p.author_id = u.id
            WHERE p.id = $1
            ",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn get_by_parent_id(&self, parent_id: i32) -> Result<Vec<PostInfo>, RepoError> {
        sqlx::query_as!(
            PostInfo,
            "
            SELECT p.*, u.username AS author_username
            FROM post p
            LEFT JOIN users u ON p.author_id = u.id
            WHERE p.parent_id = $1
            ORDER BY p.created_at DESC
            ",
            parent_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn user_posts_by_id(&self, author_id: i32) -> Result<Vec<PostInfo>, RepoError> {
        sqlx::query_as!(
            PostInfo,
            "
            SELECT p.*, u.username AS author_username
            FROM post p
            LEFT JOIN users u ON p.author_id = u.id
            WHERE p.author_id = $1
            ORDER BY p.created_at DESC
            ",
            author_id,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn user_posts_by_username(
        &self,
        author_username: &str,
    ) -> Result<Vec<PostInfo>, RepoError> {
        sqlx::query_as!(
            PostInfo,
            "
            SELECT p.*, $1 AS author_username
            FROM post p
            WHERE p.author_id = (
                SELECT id FROM users
                WHERE username = $1
            )
            ORDER BY p.created_at DESC
            ",
            author_username,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn all_friend_posts(&self, user_id: i32) -> Result<Vec<PostInfo>, RepoError> {
        sqlx::query_as!(
            PostInfo,
            "
            SELECT p.*, u.username AS author_username
            FROM post p
            LEFT JOIN users u ON p.author_id = u.id
            JOIN (
                SELECT
                    CASE
                        WHEN f.first_id = $1 THEN f.second_id
                        ELSE f.first_id
                    END AS friend_id
                FROM friendships f
                WHERE f.confirmed AND (f.first_id = $1 OR f.second_id = $1)
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
        models::user::NewUser,
        test_utils::{
            seed_data::{seed_friends, seed_root_post, seed_users},
            temp_db::with_test_pool,
            time::within_one_second,
        },
    };
    use chrono::Utc;

    /// Runs the provided test with a [`PostRepo`] instance that has users and the root post seeded.
    async fn with_seeded_users_and_root_post<F, Fut>(test: F)
    where
        F: FnOnce(PostRepo, [NewUser; 4]) -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        with_test_pool(|pool| async move {
            let new_users = seed_users(pool.clone()).await;
            seed_root_post(&pool).await;
            test(PostRepo::new(pool), new_users).await;
        })
        .await;
    }

    /// Asserts that the result was a successful post insertion.
    fn assert_inserted(res: &Result<PostInsertionOutcome, RepoError>) {
        assert!(
            matches!(res, Ok(PostInsertionOutcome::Inserted)),
            "post insertion failed: {res:#?}"
        );
    }

    mod insert_new {
        use super::*;

        // TODO: test for deleted and archived parent posts once deleting and archiving posts is
        // implemented

        #[tokio::test]
        async fn reports_missing_parent_post() {
            with_seeded_users_and_root_post(|repo, _| async move {
                // Only post ID 1 exists, not 2
                assert!(matches!(
                    repo.insert_new(2, 2, "I'm an orphan").await,
                    Ok(PostInsertionOutcome::ParentMissing)
                ));
            })
            .await;
        }

        #[tokio::test]
        async fn disallows_replying_to_ones_own_post() {
            with_seeded_users_and_root_post(|repo, _| async move {
                // User 2 replies to user 1
                assert_inserted(&repo.insert_new(2, 1, "My first reply").await);
                // User 2 then replies to their own reply
                assert!(matches!(
                    repo.insert_new(2, 2, "I'm so good at replying").await,
                    Ok(PostInsertionOutcome::SelfReply)
                ));
                // The violating post should not have been created
                assert!(matches!(repo.get_by_id(3).await, Ok(None)));
            })
            .await;
        }

        #[tokio::test]
        async fn rejects_multiple_replies_to_the_same_post_by_the_same_user() {
            with_seeded_users_and_root_post(|repo, _| async move {
                // First reply is valid
                assert_inserted(&repo.insert_new(2, 1, "My first reply").await);
                // Second reply to the same post by the same user is invalid
                assert!(matches!(
                    repo.insert_new(2, 1, "Oh no, replying again").await,
                    Err(RepoError::UniqueViolation(v)) if v == "post_author_parent_unique"
                ));
                // The violating post should not have been created
                assert!(matches!(repo.get_by_id(3).await, Ok(None)));
            })
            .await;
        }

        #[tokio::test]
        async fn allows_multiple_replies_if_the_user_or_parent_is_different() {
            with_seeded_users_and_root_post(|repo, _| async move {
                // First reply
                assert_inserted(&repo.insert_new(2, 1, "My first reply").await);
                // Same parent post as the first reply, but from a different user
                assert_inserted(
                    &repo
                        .insert_new(3, 1, "I'm also replying to this post")
                        .await,
                );
                // Same user as the first reply, but a different parent post
                assert_inserted(&repo.insert_new(2, 3, "I'm replying to your reply").await);
            })
            .await;
        }
    }

    #[tokio::test]
    async fn sets_and_gets_correct_data() {
        with_seeded_users_and_root_post(|repo, _| async move {
            // The root post has ID 1, so start from 2
            let post_body_2 = "Hello everyone and welcome to my post.\n\
                               This is a test post where I just write\n\
                               something that makes sense for testing.";
            let post_body_3 = "Some posts might have \n\
                               some \t strange spacing in       them \t\
                               \nbut everything should still work\n \n\nfine   \n";
            let post_body_4 = "日本語の文字も使えるはずなので確認しておきましょう！";

            // All three posts should be successfully inserted
            assert_inserted(&repo.insert_new(4, 1, post_body_2).await);
            assert_inserted(&repo.insert_new(3, 2, post_body_3).await);
            assert_inserted(&repo.insert_new(2, 2, post_body_4).await);

            let post2 = repo
                .get_by_id(2)
                .await
                .expect("failed to get post 2")
                .expect("post 2 was None");
            let post3 = repo
                .get_by_id(3)
                .await
                .expect("failed to get post 3")
                .expect("post 3 was None");
            let post4 = repo
                .get_by_id(4)
                .await
                .expect("failed to get post 4")
                .expect("post 4 was None");

            assert_eq!(post2.author_id, Some(4));
            assert_eq!(post2.parent_id, Some(1));
            assert_eq!(post2.body, Some(post_body_2.to_string()));
            assert!(within_one_second(post2.created_at, Utc::now()));
            assert!(post2.edited_at.is_none());
            assert!(post2.archived_at.is_none());
            assert!(post2.deleted_at.is_none());

            assert_eq!(post3.author_id, Some(3));
            assert_eq!(post3.parent_id, Some(2));
            assert_eq!(post3.body, Some(post_body_3.to_string()));
            assert!(within_one_second(post3.created_at, Utc::now()));
            assert!(post3.edited_at.is_none());
            assert!(post3.archived_at.is_none());
            assert!(post3.deleted_at.is_none());

            assert_eq!(post4.author_id, Some(2));
            assert_eq!(post4.parent_id, Some(2));
            assert_eq!(post4.body, Some(post_body_4.to_string()));
            assert!(within_one_second(post4.created_at, Utc::now()));
            assert!(post4.edited_at.is_none());
            assert!(post4.archived_at.is_none());
            assert!(post4.deleted_at.is_none());
        })
        .await;
    }

    #[tokio::test]
    async fn returns_none_for_nonexistent_posts() {
        with_seeded_users_and_root_post(|repo, _| async move {
            repo.insert_new(3, 1, "This post exists").await.unwrap(); // Assigned ID 2
            assert!(matches!(repo.get_by_id(3).await, Ok(None))); // No ID 3
        })
        .await;
    }

    #[tokio::test]
    async fn gets_zero_one_or_many_child_posts_of_a_parent_and_no_others() {
        with_seeded_users_and_root_post(|repo, _| async move {
            let parent_id = 2;
            // Insert post to be the parent
            repo.insert_new(4, 1, "I'm going to be a parent soon") // ID 2
                .await
                .unwrap();
            // Should not retrieve sibling
            repo.insert_new(3, 1, "I'm your sibling, not your child") // ID 3
                .await
                .unwrap();
            // No children at first
            assert!(matches!(
                repo.get_by_parent_id(parent_id).await,
                Ok(v) if v.is_empty()
            ));
            // First child
            repo.insert_new(1, parent_id, "I'm your first child") // ID 4
                .await
                .unwrap();
            // Should not retrieve grandchildren
            repo.insert_new(2, 4, "I'm your grandchild, not your child") // ID 5
                .await
                .unwrap();
            let first_child = repo.get_by_id(4).await.unwrap().unwrap();
            assert!(matches!(
                repo.get_by_parent_id(parent_id).await,
                Ok(v) if v.len() == 1 && v[0] == first_child
            ));
            // More children
            repo.insert_new(2, parent_id, "Second child here") // ID 6
                .await
                .unwrap();
            repo.insert_new(3, parent_id, "Third child here") // ID 7
                .await
                .unwrap();
            let second_child = repo.get_by_id(6).await.unwrap().unwrap();
            let third_child = repo.get_by_id(7).await.unwrap().unwrap();
            // Should be sorted in descending order of creation time
            let expected_children = vec![third_child, second_child, first_child];
            assert!(matches!(
                repo.get_by_parent_id(parent_id).await,
                Ok(v) if v == expected_children
            ));
        })
        .await;
    }

    #[tokio::test]
    async fn gets_only_posts_by_a_specific_user() {
        with_seeded_users_and_root_post(|repo, users| async move {
            assert_inserted(&repo.insert_new(3, 1, "First post by user 3").await); // Post ID 2
            assert_inserted(
                &repo
                    .insert_new(2, 2, "This post by user 2 should not come up") // Post ID 3
                    .await,
            );
            assert_inserted(
                &repo
                    .insert_new(4, 1, "This post by user 4 should not come up ") // Post ID 4
                    .await,
            );
            assert_inserted(&repo.insert_new(3, 4, "Second post by user 3").await); // Post ID 5

            let expected1 = repo.get_by_id(2).await.unwrap().unwrap();
            let expected2 = repo.get_by_id(5).await.unwrap().unwrap();

            // Should be sorted by created_at in descending order
            let expected_posts = vec![expected2, expected1];
            assert!(matches!(repo.user_posts_by_id(3).await, Ok(v) if v == expected_posts));

            // Searching by username should be the same result
            assert!(matches!(
                repo.user_posts_by_username(&users[2].username).await,
                Ok(v) if v == expected_posts
            ));
        })
        .await;
    }

    #[tokio::test]
    async fn gets_only_posts_by_friends_of_a_user() {
        with_test_pool(|pool| async move {
            seed_users(pool.clone()).await;
            seed_root_post(&pool).await;
            seed_friends(pool.clone()).await;

            let repo = PostRepo::new(pool);

            let u1p2_body = "User one post two";
            let u1p3_body = "User one post three";
            let u2p1_body = "User two post one";
            let u2p2_body = "User two post two";
            let u3p1_body = "User three post one";
            let u3p2_body = "User three post two";
            let u4p1_body = "User four post one";
            let u4p2_body = "User four post two";

            repo.insert_new(4, 1, u4p1_body).await.unwrap(); // ID 2
            repo.insert_new(3, 1, u3p1_body).await.unwrap(); // ID 3
            repo.insert_new(2, 1, u2p1_body).await.unwrap(); // ID 4
            repo.insert_new(1, 2, u1p2_body).await.unwrap(); // ID 5
            repo.insert_new(4, 3, u4p2_body).await.unwrap(); // ID 6
            repo.insert_new(3, 2, u3p2_body).await.unwrap(); // ID 7
            repo.insert_new(2, 2, u2p2_body).await.unwrap(); // ID 8
            repo.insert_new(1, 3, u1p3_body).await.unwrap(); // ID 9

            let u2p1 = repo.get_by_id(4).await.unwrap().unwrap();
            let u2p2 = repo.get_by_id(8).await.unwrap().unwrap();
            let u3p1 = repo.get_by_id(3).await.unwrap().unwrap();
            let u3p2 = repo.get_by_id(7).await.unwrap().unwrap();
            let u4p1 = repo.get_by_id(2).await.unwrap().unwrap();
            let u4p2 = repo.get_by_id(6).await.unwrap().unwrap();

            let u1_friend_posts = repo.all_friend_posts(1).await.unwrap();
            let u2_friend_posts = repo.all_friend_posts(2).await.unwrap();
            let u3_friend_posts = repo.all_friend_posts(3).await.unwrap();
            let u4_friend_posts = repo.all_friend_posts(4).await.unwrap();

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
