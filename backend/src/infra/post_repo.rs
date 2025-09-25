use crate::{
    domain::{RepoError, post::PostRepo},
    models::post::Post,
};
use sqlx::PgExecutor;

pub struct PgPostRepo;

#[async_trait::async_trait]
impl PostRepo for PgPostRepo {
    async fn insert_new(
        &self,
        exec: impl PgExecutor<'_>,
        author_id: i32,
        parent_id: i32,
        body: &str,
    ) -> Result<(), RepoError> {
        sqlx::query!(
            "INSERT INTO post (author_id, parent_id, body) VALUES ($1, $2, $3::text)",
            author_id,
            parent_id,
            body
        )
        .execute(exec)
        .await
        .map_err(Into::into)
        .map(|_| ())
    }

    async fn get_by_id(
        &self,
        exec: impl PgExecutor<'_>,
        id: i32,
    ) -> Result<Option<Post>, RepoError> {
        sqlx::query_as!(Post, "SELECT * FROM post WHERE id = $1", id)
            .fetch_optional(exec)
            .await
            .map_err(Into::into)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::{
//         models::post::Post,
//         test_utils::{seed_data::with_seeded_users_and_root_post, time::within_five_seconds},
//     };
//     use chrono::Utc;
//     use sqlx::PgExecutor;
//
//     async fn expect_post_by_id(exec: impl PgExecutor<'_>, id: i32) -> Post {
//         sqlx::query_as!(Post, "SELECT * FROM post WHERE id = $1", id)
//             .fetch_one(exec)
//             .await
//             .expect("failed to get post")
//     }
//
//     /// Asserts that the result was a successful post insertion.
//     fn assert_inserted(res: &Result<PostInsertionOutcome, RepoError>) {
//         assert!(
//             matches!(res, Ok(PostInsertionOutcome::Inserted)),
//             "post insertion failed: {res:#?}"
//         );
//     }
//
//     mod insert_new {
//         use super::*;
//
//         // TODO: test for deleted and archived parent posts once deleting and archiving posts is
//         // implemented
//
//         #[tokio::test]
//         async fn reports_missing_parent_post() {
//             with_seeded_users_and_root_post(|_, repo, _| async move {
//                 // Only post ID 1 exists, not 2
//                 assert!(matches!(
//                     repo.insert_new(2, 2, "I'm an orphan").await,
//                     Ok(PostInsertionOutcome::ParentMissing)
//                 ));
//             })
//             .await;
//         }
//
//         #[tokio::test]
//         async fn disallows_replying_to_ones_own_post() {
//             with_seeded_users_and_root_post(|pool, repo, _| async move {
//                 // User 2 replies to user 1
//                 assert_inserted(&repo.insert_new(2, 1, "My first reply").await);
//                 // User 2 then replies to their own reply
//                 assert!(matches!(
//                     repo.insert_new(2, 2, "I'm so good at replying").await,
//                     Ok(PostInsertionOutcome::SelfReply)
//                 ));
//                 // The violating post should not have been created
//                 assert!(
//                     tokio::spawn(async move {
//                         expect_post_by_id(&pool, 3).await;
//                     })
//                     .await
//                     .is_err_and(|e| e.is_panic())
//                 );
//             })
//             .await;
//         }
//
//         #[tokio::test]
//         async fn rejects_multiple_replies_to_the_same_post_by_the_same_user() {
//             with_seeded_users_and_root_post(|pool, repo, _| async move {
//                 // First reply is valid
//                 assert_inserted(&repo.insert_new(2, 1, "My first reply").await);
//                 // Second reply to the same post by the same user is invalid
//                 assert!(matches!(
//                     repo.insert_new(2, 1, "Oh no, replying again").await,
//                     Err(RepoError::UniqueViolation(v)) if v == "post_author_parent_unique"
//                 ));
//                 // The violating post should not have been created
//                 assert!(
//                     tokio::spawn(async move {
//                         expect_post_by_id(&pool, 3).await;
//                     })
//                     .await
//                     .is_err_and(|e| e.is_panic())
//                 );
//             })
//             .await;
//         }
//
//         #[tokio::test]
//         async fn allows_multiple_replies_if_the_user_or_parent_is_different() {
//             with_seeded_users_and_root_post(|_, repo, _| async move {
//                 // First reply
//                 assert_inserted(&repo.insert_new(2, 1, "My first reply").await);
//                 // Same parent post as the first reply, but from a different user
//                 assert_inserted(
//                     &repo
//                         .insert_new(3, 1, "I'm also replying to this post")
//                         .await,
//                 );
//                 // Same user as the first reply, but a different parent post
//                 assert_inserted(&repo.insert_new(2, 3, "I'm replying to your reply").await);
//             })
//             .await;
//         }
//     }
//
//     #[tokio::test]
//     async fn inserts_and_sets_correct_data() {
//         with_seeded_users_and_root_post(|pool, repo, _| async move {
//             // The root post has ID 1, so start from 2
//             let post_body_2 = "Hello everyone and welcome to my post.\n\
//                                This is a test post where I just write\n\
//                                something that makes sense for testing.";
//             let post_body_3 = "Some posts might have \n\
//                                some \t strange spacing in       them \t\
//                                \nbut everything should still work\n \n\nfine   \n";
//             let post_body_4 = "日本語の文字も使えるはずなので確認しておきましょう！";
//
//             // All three posts should be successfully inserted
//             assert_inserted(&repo.insert_new(4, 1, post_body_2).await);
//             assert_inserted(&repo.insert_new(3, 2, post_body_3).await);
//             assert_inserted(&repo.insert_new(2, 2, post_body_4).await);
//
//             let post2 = expect_post_by_id(&pool, 2).await;
//             let post3 = expect_post_by_id(&pool, 3).await;
//             let post4 = expect_post_by_id(&pool, 4).await;
//
//             assert_eq!(post2.id, 2);
//             assert_eq!(post2.author_id, Some(4));
//             assert_eq!(post2.parent_id, Some(1));
//             assert_eq!(post2.body, Some(post_body_2.to_string()));
//             assert!(within_five_seconds(post2.created_at, Utc::now()));
//             assert!(post2.edited_at.is_none());
//             assert!(post2.archived_at.is_none());
//             assert!(post2.deleted_at.is_none());
//
//             assert_eq!(post3.id, 3);
//             assert_eq!(post3.author_id, Some(3));
//             assert_eq!(post3.parent_id, Some(2));
//             assert_eq!(post3.body, Some(post_body_3.to_string()));
//             assert!(within_five_seconds(post3.created_at, Utc::now()));
//             assert!(post3.edited_at.is_none());
//             assert!(post3.archived_at.is_none());
//             assert!(post3.deleted_at.is_none());
//
//             assert_eq!(post4.id, 4);
//             assert_eq!(post4.author_id, Some(2));
//             assert_eq!(post4.parent_id, Some(2));
//             assert_eq!(post4.body, Some(post_body_4.to_string()));
//             assert!(within_five_seconds(post4.created_at, Utc::now()));
//             assert!(post4.edited_at.is_none());
//             assert!(post4.archived_at.is_none());
//             assert!(post4.deleted_at.is_none());
//         })
//         .await;
//     }
// }
