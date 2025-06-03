use super::insertion_error::InsertionError;
use crate::{
    domain::content::repository::PostStore,
    models::post::{Post, PostWithPromptRow},
    technical_error::TechnicalError,
};
use spur_shared::models::PostWithPrompt;

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
        prompt_id: i32,
        body: &str,
    ) -> Result<i32, InsertionError> {
        let rec = sqlx::query!(
            "INSERT INTO posts (author_id, prompt_id, body) VALUES ($1, $2, $3) RETURNING id",
            author_id,
            prompt_id,
            body,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(rec.id)
    }

    async fn get_by_id(&self, id: i32) -> Result<Option<Post>, TechnicalError> {
        let maybe_post = sqlx::query_as!(Post, "SELECT * FROM posts WHERE posts.id = $1", id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(maybe_post)
    }

    async fn single_user_posts(
        &self,
        author_id: i32,
    ) -> Result<Vec<PostWithPrompt>, TechnicalError> {
        let posts = sqlx::query_as!(
            PostWithPromptRow,
            "
            SELECT
                posts.id AS post_id,
                u1.username AS post_author_username,
                posts.body AS post_body,

                prompts.id AS prompt_id,
                u2.username AS prompt_author_username,
                prompts.body AS prompt_body
            FROM posts

            JOIN prompts ON posts.prompt_id = prompts.id
            JOIN users u1 ON posts.author_id = u1.id
            JOIN users u2 ON prompts.author_id = u2.id

            WHERE posts.author_id = $1
            ORDER BY posts.created_at DESC
            ",
            author_id,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(posts.into_iter().map(Into::into).collect())
    }

    async fn all_friend_posts(&self, user_id: i32) -> Result<Vec<PostWithPrompt>, TechnicalError> {
        let posts = sqlx::query_as!(
            PostWithPromptRow,
            "
            SELECT
                posts.id AS post_id,
                u1.username AS post_author_username,
                posts.body AS post_body,

                prompts.id AS prompt_id,
                u2.username AS prompt_author_username,
                prompts.body AS prompt_body
            FROM posts

            JOIN (
                SELECT
                    CASE
                        WHEN f.first_id = $1 THEN f.second_id
                        ELSE f.first_id
                    END AS friend_id
                FROM friendships f
                WHERE f.confirmed
                AND ($1 = f.first_id OR $1 = f.second_id)
            ) AS friends ON posts.author_id = friends.friend_id

            JOIN prompts ON posts.prompt_id = prompts.id
            JOIN users u1 ON posts.author_id = u1.id
            JOIN users u2 ON prompts.author_id = u2.id

            ORDER BY posts.created_at DESC
            ",
            user_id,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(posts.into_iter().map(Into::into).collect())
    }
}
