use super::insertion_error::InsertionError;
use crate::{
    domain::content::repository::PostStore, models::post::Post, technical_error::TechnicalError,
};

pub struct PostRepo {
    pool: sqlx::PgPool,
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

    async fn get_user_posts(&self, author_id: i32) -> Result<Vec<Post>, TechnicalError> {
        let posts = sqlx::query_as!(
            Post,
            "SELECT * FROM posts WHERE posts.author_id = $1",
            author_id,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(posts)
    }

    async fn get_friend_posts(&self, user_id: i32) -> Result<Vec<Post>, TechnicalError> {
        let posts = sqlx::query_as!(
            Post,
            "
            SELECT p.*
            FROM posts p
            JOIN (
                SELECT
                    CASE
                        WHEN f.first_id = $1 THEN f.second_id
                        ELSE f.first_id
                    END AS friend_id
                FROM friendships f
                WHERE f.confirmed
                AND ($1 = f.first_id OR $1 = f.second_id)
            ) AS friends
            ON p.author_id = friends.friend_id
            ORDER BY p.created_at DESC;
            ",
            user_id,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(posts)
    }
}
