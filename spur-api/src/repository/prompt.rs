use super::insertion_error::InsertionError;
use crate::{
    domain::content::repository::PromptStore, models::prompt::Prompt,
    technical_error::TechnicalError,
};
use spur_shared::models::PromptWithAuthor;

pub struct PromptRepo {
    pool: sqlx::PgPool,
}

impl PromptRepo {
    pub const fn new(pool: sqlx::PgPool) -> Self { Self { pool } }
}

#[async_trait::async_trait]
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

    async fn single_user_prompts(
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
            ORDER BY prompts.created_at DESC
            ",
            user_id,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(prompts)
    }

    async fn all_friend_prompts(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::friendship::{repository::FriendshipStore, user_id_pair::UserIdPair},
        repository::friendship::FriendshipRepo,
        test_utils::{must_seed_users, with_test_pool, within_one_second},
    };
    use chrono::Utc;

    #[tokio::test]
    async fn inserts_and_gets_correct_data() {
        with_test_pool(|pool| async move {
            // Authors must be existing users
            must_seed_users(pool.clone()).await;

            let repo = PromptRepo::new(pool);

            let prompt_body_1 = "This is a test prompt. This is only a test prompt";
            let prompt_body_2 = "What was it like when you wrote your first prompt?";

            let prompt_id_1 = repo
                .insert_new(1, prompt_body_1)
                .await
                .expect("failed to insert prompt 1");
            let prompt_id_2 = repo
                .insert_new(2, prompt_body_2)
                .await
                .expect("failed to insert prompt 2");

            let prompt1 = repo
                .get_by_id(prompt_id_1)
                .await
                .expect("failed to get prompt 1")
                .expect("prompt 1 was None");
            let prompt2 = repo
                .get_by_id(prompt_id_2)
                .await
                .expect("failed to get prompt 2")
                .expect("prompt 2 was None");

            assert_eq!(prompt1.id, prompt_id_1);
            assert_eq!(prompt1.author_id, 1);
            assert_eq!(prompt1.body, prompt_body_1);
            assert!(within_one_second(prompt1.created_at, Utc::now()));

            assert_eq!(prompt2.id, prompt_id_2);
            assert_eq!(prompt2.author_id, 2);
            assert_eq!(prompt2.body, prompt_body_2);
            assert!(within_one_second(prompt2.created_at, Utc::now()));
        })
        .await;
    }

    #[tokio::test]
    async fn returns_none_for_nonexistent_prompts() {
        with_test_pool(|pool| async move {
            must_seed_users(pool.clone()).await;
            let repo = PromptRepo::new(pool);

            repo.insert_new(1, "Anything here")
                .await
                .expect("failed to insert prompt 1");
            repo.insert_new(2, "Anything there")
                .await
                .expect("failed to insert prompt 2");

            assert!(matches!(repo.get_by_id(3).await, Ok(None)));
            assert!(matches!(repo.get_by_id(4).await, Ok(None)));
        })
        .await;
    }

    #[tokio::test]
    async fn rejects_duplicate_prompts_from_the_same_author() {
        with_test_pool(|pool| async move {
            must_seed_users(pool.clone()).await;
            let repo = PromptRepo::new(pool);

            let prompt_body = "Repetition legitimizes";
            repo.insert_new(1, prompt_body)
                .await
                .expect("failed to insert prompt the first time");

            let result = repo.insert_new(1, prompt_body).await;
            assert!(matches!(result, Err(InsertionError::UniqueViolation(_))));
        })
        .await;
    }

    #[tokio::test]
    async fn allows_duplicate_prompts_from_different_authors() {
        with_test_pool(|pool| async move {
            must_seed_users(pool.clone()).await;
            let repo = PromptRepo::new(pool);

            let prompt_body = "Somebody said repetition legitimizes";
            repo.insert_new(1, prompt_body)
                .await
                .expect("failed to insert prompt the first time");

            let result = repo.insert_new(2, prompt_body).await;
            assert!(matches!(result, Ok(2)));
        })
        .await;
    }

    #[tokio::test]
    async fn gets_only_prompts_by_a_specific_user() {
        with_test_pool(|pool| async move {
            let (_, user2, _, _) = must_seed_users(pool.clone()).await;
            let repo = PromptRepo::new(pool);

            let user2prompt1 = "Hello from the database";
            let user2prompt2 = "Hope you're feeling creative";

            repo.insert_new(1, "Should not get this one").await.unwrap();
            repo.insert_new(1, "Should not get this one either")
                .await
                .unwrap();
            let user2prompt1id = repo.insert_new(2, user2prompt1).await.unwrap();
            let user2prompt2id = repo.insert_new(2, user2prompt2).await.unwrap();
            repo.insert_new(3, "Finally, this one should not come up either")
                .await
                .unwrap();

            let mut got_prompts = repo
                .single_user_prompts(2)
                .await
                .expect("failed to get user prompts");

            // Reverse because they should have been sorted by created_at in descending order
            got_prompts.reverse();

            let expected1 = PromptWithAuthor {
                id: user2prompt1id,
                author_username: user2.username.clone(),
                body: user2prompt1.to_string(),
            };

            let expected2 = PromptWithAuthor {
                id: user2prompt2id,
                author_username: user2.username,
                body: user2prompt2.to_string(),
            };

            assert_eq!(got_prompts.len(), 2);
            assert_eq!(got_prompts[0], expected1);
            assert_eq!(got_prompts[1], expected2);
        })
        .await;
    }

    #[tokio::test]
    async fn gets_only_prompts_by_friends_of_a_user() {
        with_test_pool(|pool| async move {
            let (_, u2, u3, u4) = must_seed_users(pool.clone()).await;
            let friendship_repo = FriendshipRepo::new(pool.clone());
            let prompt_repo = PromptRepo::new(pool);

            /*
             * 1 & 2 => no relation
             * 1 & 3 => requested, unconfirmed
             * 1 & 4 => no relation
             * 2 & 3 => confirmed friends
             * 2 & 4 => confirmed friends
             * 3 & 4 => requested, unconfirmed
             *
             * 1 => no friends
             * 2 => friends with 3 and 4
             * 3 => friends with 2
             * 4 => friends with 2
             */
            let two_and_three = UserIdPair::new(2, 3).unwrap();
            let two_and_four = UserIdPair::new(4, 2).unwrap();
            friendship_repo
                .new_request(&two_and_three, 2)
                .await
                .unwrap();
            friendship_repo.new_request(&two_and_four, 4).await.unwrap();
            friendship_repo
                .accept_request(&two_and_three)
                .await
                .unwrap();
            friendship_repo.accept_request(&two_and_four).await.unwrap();
            friendship_repo
                .new_request(&UserIdPair::new(1, 3).unwrap(), 3)
                .await
                .unwrap();
            friendship_repo
                .new_request(&UserIdPair::new(4, 3).unwrap(), 3)
                .await
                .unwrap();

            let u1p1 = "User one prompt one";
            let u1p2 = "User one prompt two";
            let u2p1 = "User two prompt one";
            let u2p2 = "User two prompt two";
            let u3p1 = "User three prompt one";
            let u3p2 = "User three prompt two";
            let u4p1 = "User four prompt one";
            let u4p2 = "User four prompt two";

            prompt_repo.insert_new(1, u1p1).await.unwrap();
            prompt_repo.insert_new(1, u1p2).await.unwrap();
            let u2p1id = prompt_repo.insert_new(2, u2p1).await.unwrap();
            let u2p2id = prompt_repo.insert_new(2, u2p2).await.unwrap();
            let u3p1id = prompt_repo.insert_new(3, u3p1).await.unwrap();
            let u3p2id = prompt_repo.insert_new(3, u3p2).await.unwrap();
            let u4p1id = prompt_repo.insert_new(4, u4p1).await.unwrap();
            let u4p2id = prompt_repo.insert_new(4, u4p2).await.unwrap();

            let u2p1_expected = PromptWithAuthor {
                id: u2p1id,
                author_username: u2.username.clone(),
                body: u2p1.to_string(),
            };

            let u2p2_expected = PromptWithAuthor {
                id: u2p2id,
                author_username: u2.username,
                body: u2p2.to_string(),
            };

            let u3p1_expected = PromptWithAuthor {
                id: u3p1id,
                author_username: u3.username.clone(),
                body: u3p1.to_string(),
            };

            let u3p2_expected = PromptWithAuthor {
                id: u3p2id,
                author_username: u3.username,
                body: u3p2.to_string(),
            };

            let u4p1_expected = PromptWithAuthor {
                id: u4p1id,
                author_username: u4.username.clone(),
                body: u4p1.to_string(),
            };

            let u4p2_expected = PromptWithAuthor {
                id: u4p2id,
                author_username: u4.username,
                body: u4p2.to_string(),
            };

            let u1_friend_prompts = prompt_repo.all_friend_prompts(1).await.unwrap();
            let mut u2_friend_prompts = prompt_repo.all_friend_prompts(2).await.unwrap();
            let mut u3_friend_prompts = prompt_repo.all_friend_prompts(3).await.unwrap();
            let mut u4_friend_prompts = prompt_repo.all_friend_prompts(4).await.unwrap();

            // Reverse because they should have been sorted by created_at in descending order
            u2_friend_prompts.reverse();
            u3_friend_prompts.reverse();
            u4_friend_prompts.reverse();

            // 1 has no friends
            assert!(u1_friend_prompts.is_empty());

            // 2 is friends with both 3 and 4
            assert_eq!(u2_friend_prompts.len(), 4);
            assert_eq!(u2_friend_prompts[0], u3p1_expected);
            assert_eq!(u2_friend_prompts[1], u3p2_expected);
            assert_eq!(u2_friend_prompts[2], u4p1_expected);
            assert_eq!(u2_friend_prompts[3], u4p2_expected);

            // 3 is only friends with 2
            assert_eq!(u3_friend_prompts.len(), 2);
            assert_eq!(u3_friend_prompts[0], u2p1_expected);
            assert_eq!(u3_friend_prompts[1], u2p2_expected);

            // 4 is also only friends with 2
            assert_eq!(u4_friend_prompts.len(), 2);
            assert_eq!(u4_friend_prompts[0], u2p1_expected);
            assert_eq!(u4_friend_prompts[1], u2p2_expected);
        })
        .await;
    }
}
