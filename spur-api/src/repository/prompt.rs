use super::insertion_error::InsertionError;
use crate::{domain::prompt::PromptStore, models::prompt::Prompt, technical_error::TechnicalError};
use spur_shared::models::PromptWithAuthor;

pub struct PromptRepo {
    pool: sqlx::PgPool,
}

impl PromptStore for PromptRepo {
    async fn insert_new(&self, author_id: i32, body: &str) -> Result<i32, InsertionError> {
        let rec = sqlx::query!(
            "INSERT INTO prompts (author_id, body) VALUES ($1, $2) RETURNING id",
            author_id,
            body,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(rec.id)
    }

    async fn get_by_id(&self, id: i32) -> Result<Option<Prompt>, TechnicalError> {
        let maybe_prompt = sqlx::query_as!(
            Prompt,
            "SELECT id, author_id, body, created_at FROM prompts WHERE prompts.id = $1",
            id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(maybe_prompt)
    }

    async fn get_user_prompts(
        &self,
        user_id: i32,
    ) -> Result<Vec<PromptWithAuthor>, TechnicalError> {
        let prompts = sqlx::query_as!(
            PromptWithAuthor,
            "
            SELECT
                prompts.id, 
                users.username AS author_username,
                prompts.body
            FROM prompts
            JOIN users ON prompts.author_id = users.id
            WHERE prompts.author_id = $1
            ",
            user_id,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(prompts)
    }

    async fn get_friend_prompts(
        &self,
        user_id: i32,
    ) -> Result<Vec<PromptWithAuthor>, TechnicalError> {
        let prompts = sqlx::query_as!(
            PromptWithAuthor,
            "
            SELECT
                p.id, 
                u.username AS author_username,
                p.body
            FROM prompts p
            JOIN users u ON p.author_id = u.id
            JOIN (
                SELECT
                    CASE
                        WHEN f.first_id = $1 THEN f.second_id
                        ELSE f.first_id
                    END AS friend_id
                FROM friendships f
                WHERE f.confirmed AND (f.first_id = $1 OR f.second_id = $1)
            ) AS friends ON p.author_id = friends.friend_id
            ORDER BY p.created_at DESC
            ",
            user_id,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(prompts)
    }
}
