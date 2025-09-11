use crate::{
    models::post::PostWithAuthor,
    read_models::{PostWithAuthorRead, ReadError},
};
use sqlx::PgPool;

pub struct PgPostWithAuthorRead {
    pool: PgPool,
}

impl PgPostWithAuthorRead {
    pub const fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait::async_trait]
impl PostWithAuthorRead for PgPostWithAuthorRead {
    async fn by_post_id(&self, id: i32) -> Result<PostWithAuthor, ReadError> {
        sqlx::query_as!(
            PostWithAuthor,
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
        .and_then(|maybe_post| {
            maybe_post.ok_or_else(|| ReadError::NotFound(String::from("post not found")))
        })
    }

    async fn by_parent(&self, parent_id: i32) -> Result<Vec<PostWithAuthor>, ReadError> {
        sqlx::query_as!(
            PostWithAuthor,
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

    async fn by_author(&self, author_id: i32) -> Result<Vec<PostWithAuthor>, ReadError> {
        sqlx::query_as!(
            PostWithAuthor,
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

    async fn by_author_username(
        &self,
        author_username: &str,
    ) -> Result<Vec<PostWithAuthor>, ReadError> {
        sqlx::query_as!(
            PostWithAuthor,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{domain::post::PostStore, test_utils::seed_data::with_seeded_users_and_root_post};
    use chrono::Utc;

    #[tokio::test]
    async fn gets_an_existing_post() {
        with_seeded_users_and_root_post(|pool, repo, users| async move {
            let read = PgPostWithAuthorRead::new(pool);
            let body = "This post exists!";
            repo.insert_new(2, 1, body).await.unwrap();
            let actual = read.by_post_id(2).await;
            let expected = PostWithAuthor {
                id: 2,
                author_id: Some(2),
                parent_id: Some(1),
                body: Some(String::from(body)),
                created_at: Utc::now(),
                edited_at: None,
                archived_at: None,
                deleted_at: None,
                author_username: Some(users[1].username.clone()),
            };
            assert!(matches!(actual, Ok(p) if p == expected));
        })
        .await;
    }

    #[tokio::test]
    async fn errors_for_nonexistent_post() {
        with_seeded_users_and_root_post(|pool, repo, _| async move {
            let read = PgPostWithAuthorRead::new(pool);
            repo.insert_new(2, 1, "This post exists!").await.unwrap();
            let actual = read.by_post_id(3).await; // Only posts 1 and 2 exist
            assert!(matches!(
                actual,
                Err(ReadError::NotFound(s)) if s == "post not found"
            ));
        })
        .await;
    }

    #[tokio::test]
    async fn gets_zero_one_or_many_child_posts_of_a_parent_and_no_others() {
        with_seeded_users_and_root_post(|pool, repo, _| async move {
            let read = PgPostWithAuthorRead::new(pool);

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
                read.by_parent(parent_id).await,
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
            let first_child = read.by_post_id(4).await.unwrap();
            assert!(matches!(
                read.by_parent(parent_id).await,
                Ok(v) if v.len() == 1 && v[0] == first_child
            ));
            // More children
            repo.insert_new(2, parent_id, "Second child here") // ID 6
                .await
                .unwrap();
            repo.insert_new(3, parent_id, "Third child here") // ID 7
                .await
                .unwrap();
            let second_child = read.by_post_id(6).await.unwrap();
            let third_child = read.by_post_id(7).await.unwrap();
            // Should be sorted in descending order of creation time
            let expected_children = vec![third_child, second_child, first_child];
            assert!(matches!(
                read.by_parent(parent_id).await,
                Ok(v) if v == expected_children
            ));
        })
        .await;
    }

    #[tokio::test]
    async fn gets_only_posts_by_a_specific_user() {
        with_seeded_users_and_root_post(|pool, repo, users| async move {
            let read = PgPostWithAuthorRead::new(pool);

            repo.insert_new(3, 1, "First post by user 3").await.unwrap(); // Post ID 2
            repo.insert_new(2, 2, "This post by user 2 should not come up") // Post ID 3
                .await
                .unwrap();
            repo.insert_new(4, 1, "This post by user 4 should not come up ") // Post ID 4
                .await
                .unwrap();
            repo.insert_new(3, 4, "Second post by user 3") // Post ID 5
                .await
                .unwrap();

            let expected1 = read.by_post_id(2).await.unwrap();
            let expected2 = read.by_post_id(5).await.unwrap();

            // Should be sorted by created_at in descending order
            let expected_posts = vec![expected2, expected1];
            assert!(matches!(read.by_author(3).await, Ok(v) if v == expected_posts));

            // Searching by username should be the same result
            assert!(matches!(
                read.by_author_username(&users[2].username).await,
                Ok(v) if v == expected_posts
            ));
        })
        .await;
    }
}
