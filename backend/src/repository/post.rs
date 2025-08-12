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
                SELECT $1, $2, $3
                FROM parent_status
                WHERE status = 'ok'
            )
            SELECT status FROM parent_status
            ",
            author_id,
            parent_id,
            body,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into) // Technical errors and unique violations
        .and_then(|status| {
            // Business rules enforced in SQL
            status.map_or(Ok(PostInsertionOutcome::ParentMissing), |s| {
                match s.as_str() {
                    "deleted" => Ok(PostInsertionOutcome::ParentDeleted),
                    "archived" => Ok(PostInsertionOutcome::ParentArchived),
                    "self_reply" => Ok(PostInsertionOutcome::SelfReply),
                    "ok" => Ok(PostInsertionOutcome::Inserted),
                    _ => {
                        Err(anyhow!("Unexpected insertion status despite hardcoded strings").into())
                    }
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
            WHERE p.id = $1;
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
            WHERE p.parent_id = $1;
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
            ORDER BY created_at DESC
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
            ORDER BY created_at DESC
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

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::{
//         models::post::PostWithPrompt,
//         test_utils::{
//             seed_data::{seed_friends, seed_prompts, seed_users},
//             temp_db::with_test_pool,
//             within_one_second,
//         },
//         utils::vec_into,
//     };
//     use chrono::Utc;
//
//     #[tokio::test]
//     async fn inserts_and_gets_correct_data() {
//         with_test_pool(|pool| async move {
//             // Relevant users and prompts must exist for posts to work
//             seed_users(pool.clone()).await;
//             let prompts = seed_prompts(pool.clone()).await;
//
//             let repo = PostRepo::new(pool);
//
//             let post_body_1 = "Hello everyone and welcome to my post.\n\
//                                This is a test post where I just write\n\
//                                something that makes sense for testing.";
//             let post_body_2 = "Some posts might have \n\
//                                some \t strange spacing in       them \t\
//                                \nbut everything should still work\n \n\nfine   \n";
//             let post_body_3 = "日本語の文字も使えるはずなので確認しておきましょう！";
//
//             let post_info_1 = repo
//                 .insert_new(4, prompts[2].id, post_body_1)
//                 .await
//                 .expect("failed to insert post 1");
//             let post_info_2 = repo
//                 .insert_new(3, prompts[1].id, post_body_2)
//                 .await
//                 .expect("failed to insert post 2");
//             let post_info_3 = repo
//                 .insert_new(2, prompts[2].id, post_body_3)
//                 .await
//                 .expect("failed to insert post 3");
//
//             let post1 = repo
//                 .get_by_id(post_info_1.id)
//                 .await
//                 .expect("failed to get post 1")
//                 .expect("post 1 was None");
//             let post2 = repo
//                 .get_by_id(post_info_2.id)
//                 .await
//                 .expect("failed to get post 2")
//                 .expect("post 2 was None");
//             let post3 = repo
//                 .get_by_id(post_info_3.id)
//                 .await
//                 .expect("failed to get post 3")
//                 .expect("post 3 was None");
//
//             assert_eq!(post1.author_id, 4);
//             assert_eq!(post1.prompt_id, prompts[2].id);
//             assert_eq!(post1.body, post_body_1);
//             assert!(within_one_second(post1.created_at, Utc::now()));
//             assert!(post1.edited_at.is_none());
//
//             assert_eq!(post2.author_id, 3);
//             assert_eq!(post2.prompt_id, prompts[1].id);
//             assert_eq!(post2.body, post_body_2);
//             assert!(within_one_second(post2.created_at, Utc::now()));
//             assert!(post2.edited_at.is_none());
//
//             assert_eq!(post3.author_id, 2);
//             assert_eq!(post3.prompt_id, prompts[2].id);
//             assert_eq!(post3.body, post_body_3);
//             assert!(within_one_second(post3.created_at, Utc::now()));
//             assert!(post3.edited_at.is_none());
//         })
//         .await;
//     }
//
//     #[tokio::test]
//     async fn rejects_multiple_posts_by_the_same_author_responding_to_the_same_prompt() {
//         with_test_pool(|pool| async move {
//             seed_users(pool.clone()).await;
//             let prompts = seed_prompts(pool.clone()).await;
//
//             let repo = PostRepo::new(pool);
//
//             repo.insert_new(2, prompts[3].id, "First post in response to this prompt")
//                 .await
//                 .expect("failed to insert the first time");
//
//             let result = repo
//                 .insert_new(
//                     2,
//                     prompts[3].id,
//                     "This could be different content and it should still get rejected",
//                 )
//                 .await;
//
//             assert!(matches!(result, Err(InsertionError::UniqueViolation(_))));
//         })
//         .await;
//     }
//
//     #[tokio::test]
//     async fn returns_none_for_nonexistent_posts() {
//         with_test_pool(|pool| async move {
//             seed_users(pool.clone()).await;
//             let prompts = seed_prompts(pool.clone()).await;
//
//             let repo = PostRepo::new(pool);
//
//             let existing_post_info = repo
//                 .insert_new(3, prompts[5].id, "This post exists")
//                 .await
//                 .unwrap();
//
//             let result = repo.get_by_id(existing_post_info.id + 1).await;
//             assert!(matches!(result, Ok(None)));
//         })
//         .await;
//     }
//
//     #[tokio::test]
//     async fn gets_only_posts_by_a_specific_user() {
//         with_test_pool(|pool| async move {
//             let users = seed_users(pool.clone()).await;
//             let prompts = seed_prompts(pool.clone()).await;
//
//             let repo = PostRepo::new(pool);
//
//             let user3post1 = "Hello this is\nmy post.";
//             let user3post2 = "This\nis also\nmy post, but it's\na different one.";
//
//             repo.insert_new(2, prompts[5].id, "This post body should not come up")
//                 .await
//                 .unwrap();
//             repo.insert_new(4, prompts[0].id, "This post body should not come up either")
//                 .await
//                 .unwrap();
//             let u3p1info = repo.insert_new(3, prompts[2].id, user3post1).await.unwrap();
//             let u3p2info = repo.insert_new(3, prompts[1].id, user3post2).await.unwrap();
//
//             let mut got_posts = repo.single_user_posts(3).await.unwrap();
//             // Reverse because they should have been sorted by created_at in descending order
//             got_posts.reverse();
//
//             let expected1 = PostWithPrompt {
//                 id: u3p1info.id,
//                 author_username: users[2].clone().username,
//                 prompt: prompts[2].clone(),
//                 body: user3post1.to_string(),
//             };
//
//             let expected2 = PostWithPrompt {
//                 id: u3p2info.id,
//                 author_username: users[2].clone().username,
//                 prompt: prompts[1].clone(),
//                 body: user3post2.to_string(),
//             };
//
//             assert_eq!(got_posts.len(), 2);
//             assert_eq!(PostWithPrompt::from(got_posts[0].clone()), expected1);
//             assert_eq!(PostWithPrompt::from(got_posts[1].clone()), expected2);
//         })
//         .await;
//     }
//
//     #[tokio::test]
//     async fn gets_only_posts_by_friends_of_a_user() {
//         with_test_pool(|pool| async move {
//             let users = seed_users(pool.clone()).await;
//             let prompts = seed_prompts(pool.clone()).await;
//             let [_, u2, u3, u4] = users;
//             seed_friends(pool.clone()).await;
//
//             let repo = PostRepo::new(pool);
//
//             let u1p1 = "User one post one";
//             let u1p2 = "User one post two";
//             let u2p1 = "User two post one";
//             let u2p2 = "User two post two";
//             let u3p1 = "User three post one";
//             let u3p2 = "User three post two";
//             let u4p1 = "User four post one";
//             let u4p2 = "User four post two";
//
//             repo.insert_new(1, prompts[7].id, u1p1).await.unwrap();
//             repo.insert_new(1, prompts[6].id, u1p2).await.unwrap();
//             let u2p1info = repo.insert_new(2, prompts[5].id, u2p1).await.unwrap();
//             let u2p2info = repo.insert_new(2, prompts[4].id, u2p2).await.unwrap();
//             let u3p1info = repo.insert_new(3, prompts[3].id, u3p1).await.unwrap();
//             let u3p2info = repo.insert_new(3, prompts[2].id, u3p2).await.unwrap();
//             let u4p1info = repo.insert_new(4, prompts[1].id, u4p1).await.unwrap();
//             let u4p2info = repo.insert_new(4, prompts[0].id, u4p2).await.unwrap();
//
//             let u2p1_expected = PostWithPrompt {
//                 id: u2p1info.id,
//                 author_username: u2.username.clone(),
//                 prompt: prompts[5].clone(),
//                 body: u2p1.to_string(),
//             };
//             let u2p2_expected = PostWithPrompt {
//                 id: u2p2info.id,
//                 author_username: u2.username,
//                 prompt: prompts[4].clone(),
//                 body: u2p2.to_string(),
//             };
//             let u3p1_expected = PostWithPrompt {
//                 id: u3p1info.id,
//                 author_username: u3.username.clone(),
//                 prompt: prompts[3].clone(),
//                 body: u3p1.to_string(),
//             };
//             let u3p2_expected = PostWithPrompt {
//                 id: u3p2info.id,
//                 author_username: u3.username,
//                 prompt: prompts[2].clone(),
//                 body: u3p2.to_string(),
//             };
//             let u4p1_expected = PostWithPrompt {
//                 id: u4p1info.id,
//                 author_username: u4.username.clone(),
//                 prompt: prompts[1].clone(),
//                 body: u4p1.to_string(),
//             };
//             let u4p2_expected = PostWithPrompt {
//                 id: u4p2info.id,
//                 author_username: u4.username,
//                 prompt: prompts[0].clone(),
//                 body: u4p2.to_string(),
//             };
//
//             let u1_friend_posts = repo.all_friend_posts(1).await.unwrap();
//             let mut u2_friend_posts: Vec<PostWithPrompt> =
//                 vec_into(repo.all_friend_posts(2).await.unwrap());
//             let mut u3_friend_posts: Vec<PostWithPrompt> =
//                 vec_into(repo.all_friend_posts(3).await.unwrap());
//             let mut u4_friend_posts: Vec<PostWithPrompt> =
//                 vec_into(repo.all_friend_posts(4).await.unwrap());
//
//             // Reverse because they should have been sorted by created_at in descending order
//             u2_friend_posts.reverse();
//             u3_friend_posts.reverse();
//             u4_friend_posts.reverse();
//
//             // 1 has no friends
//             assert!(u1_friend_posts.is_empty());
//
//             // 2 is friends with both 3 and 4
//             assert_eq!(u2_friend_posts.len(), 4);
//             assert_eq!(u2_friend_posts[0], u3p1_expected);
//             assert_eq!(u2_friend_posts[1], u3p2_expected);
//             assert_eq!(u2_friend_posts[2], u4p1_expected);
//             assert_eq!(u2_friend_posts[3], u4p2_expected);
//
//             // 3 is only friends with 2
//             assert_eq!(u3_friend_posts.len(), 2);
//             assert_eq!(u3_friend_posts[0], u2p1_expected);
//             assert_eq!(u3_friend_posts[1], u2p2_expected);
//
//             // 4 is also only friends with 2
//             assert_eq!(u4_friend_posts.len(), 2);
//             assert_eq!(u4_friend_posts[0], u2p1_expected);
//             assert_eq!(u4_friend_posts[1], u2p2_expected);
//         })
//         .await;
//     }
// }
