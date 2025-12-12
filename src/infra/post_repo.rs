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

    async fn get_by_id_exclusive(
        &self,
        exec: impl PgExecutor<'_>,
        id: i32,
    ) -> Result<Option<Post>, RepoError> {
        sqlx::query_as!(Post, "SELECT * FROM post WHERE id = $1 FOR UPDATE", id)
            .fetch_optional(exec)
            .await
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{
        seed_data::seed_users_and_root_post, time::within_five_seconds, tokio_test,
    };
    use anyhow::{Context, Result};
    use chrono::Utc;
    use sqlx::PgPool;

    #[sqlx::test]
    async fn rejects_multiple_replies_to_the_same_post_by_the_same_user(
        pool: PgPool,
    ) -> Result<()> {
        seed_users_and_root_post(&pool).await?;
        let repo = PgPostRepo;

        // First reply is valid
        repo.insert_new(&pool, 2, 1, "My first reply").await?;

        // Second reply to the same post by the same user is invalid
        assert!(matches!(
            repo.insert_new(&pool, 2, 1, "Oh no, replying again").await,
            Err(RepoError::UniqueViolation(v)) if v == "post_author_parent_unique"
        ));

        // The violating post should not have been created
        assert!(matches!(repo.get_by_id_exclusive(&pool, 3).await, Ok(None)));

        Ok(())
    }

    #[sqlx::test]
    async fn allows_multiple_replies_if_the_user_or_parent_is_different(
        pool: PgPool,
    ) -> Result<()> {
        seed_users_and_root_post(&pool).await?;
        let repo = PgPostRepo;

        repo.insert_new(&pool, 2, 1, "My first reply")
            .await
            .context("first reply")?;

        repo.insert_new(&pool, 3, 1, "I'm also replying to this post")
            .await
            .context("same parent post as the first reply, but from a different user")?;

        repo.insert_new(&pool, 2, 3, "I'm replying to your reply")
            .await
            .context("same user as the first reply, but a different parent post")?;

        Ok(())
    }

    #[sqlx::test]
    async fn rejects_empty_and_whitespace_only_post_bodies(pool: PgPool) -> Result<()> {
        seed_users_and_root_post(&pool).await?;

        // In actual use, invalid post bodies like these should have already been rejected at
        // the request validation level.
        //
        // Some of the following strings include the full-width space character '　' (different
        // from the ASCII space character).
        for empty_body in ["", " ", "   ", "　", "　　　", "\t", "\n\n", " \r\t \n"] {
            assert!(matches!(
                PgPostRepo.insert_new(&pool, 4, 1, empty_body).await,
                Err(RepoError::CheckViolation(v)) if v == "text_non_empty"
            ));
        }

        Ok(())
    }

    #[sqlx::test]
    async fn allows_post_bodies_with_non_whitespace_characters(pool: PgPool) -> Result<()> {
        seed_users_and_root_post(&pool).await?;

        for (i, non_empty_body) in [
            "hello",
            "   hello   ",
            " h e l l o ",
            "!    ",
            "世界",
            "　世　界　",
            "　　　　世界",
        ]
        .into_iter()
        .enumerate()
        {
            // Increment the parent ID with each insertion to avoid duplicate reply errors
            assert!(matches!(
                PgPostRepo
                    .insert_new(&pool, 4, (i + 1).try_into()?, non_empty_body)
                    .await,
                Ok(())
            ));
        }

        Ok(())
    }

    #[sqlx::test]
    async fn returns_none_for_missing_post(pool: PgPool) -> Result<()> {
        seed_users_and_root_post(&pool).await?;

        // Only post ID 1 exists, not 2
        assert!(matches!(
            PgPostRepo.get_by_id_exclusive(&pool, 2).await,
            Ok(None)
        ));

        Ok(())
    }

    #[sqlx::test]
    async fn sets_and_gets_correct_data(pool: PgPool) -> Result<()> {
        seed_users_and_root_post(&pool).await?;

        let repo = PgPostRepo;

        // The root post has ID 1, so start from 2
        let post_body_2 = "Hello everyone and welcome to my post.\n\
                               This is a test post where I just write\n\
                               something that makes sense for testing.";
        let post_body_3 = "Some posts might have \n\
                               some \t strange spacing in       them \t\
                               \nbut everything should still work\n \n\nfine   \n";
        let post_body_4 = "日本語の文字も使えるはずなので確認しておきましょう！";

        // All three posts should be successfully inserted
        repo.insert_new(&pool, 4, 1, post_body_2).await?;
        repo.insert_new(&pool, 3, 2, post_body_3).await?;
        repo.insert_new(&pool, 2, 2, post_body_4).await?;

        let post2 = repo
            .get_by_id_exclusive(&pool, 2)
            .await?
            .context("post 2 was unexpectedly None")?;
        let post3 = repo
            .get_by_id_exclusive(&pool, 3)
            .await?
            .context("post 3 was unexpectedly None")?;
        let post4 = repo
            .get_by_id_exclusive(&pool, 4)
            .await?
            .context("post 4 was unexpectedly None")?;

        assert_eq!(post2.id, 2);
        assert_eq!(post2.author_id, Some(4));
        assert_eq!(post2.parent_id, Some(1));
        assert_eq!(post2.body, Some(post_body_2.to_string()));
        assert!(within_five_seconds(post2.created_at, Utc::now()));
        assert!(post2.edited_at.is_none());
        assert!(post2.archived_at.is_none());
        assert!(post2.deleted_at.is_none());

        assert_eq!(post3.id, 3);
        assert_eq!(post3.author_id, Some(3));
        assert_eq!(post3.parent_id, Some(2));
        assert_eq!(post3.body, Some(post_body_3.to_string()));
        assert!(within_five_seconds(post3.created_at, Utc::now()));
        assert!(post3.edited_at.is_none());
        assert!(post3.archived_at.is_none());
        assert!(post3.deleted_at.is_none());

        assert_eq!(post4.id, 4);
        assert_eq!(post4.author_id, Some(2));
        assert_eq!(post4.parent_id, Some(2));
        assert_eq!(post4.body, Some(post_body_4.to_string()));
        assert!(within_five_seconds(post4.created_at, Utc::now()));
        assert!(post4.edited_at.is_none());
        assert!(post4.archived_at.is_none());
        assert!(post4.deleted_at.is_none());

        Ok(())
    }
}
