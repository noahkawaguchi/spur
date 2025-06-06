use crate::{
    domain::{
        content::{error::ContentError, repository::PromptStore, service::PromptManager},
        error::DomainError,
        friendship::{service::FriendshipManager, user_id_pair::UserIdPair},
    },
    repository::insertion_error::InsertionError,
    technical_error::TechnicalError,
    utils::vec_into,
};
use spur_shared::models::PromptWithAuthor;
use std::sync::Arc;

pub struct PromptSvc<S: PromptStore> {
    store: S,
    friendship_svc: Arc<dyn FriendshipManager>,
}

impl<S: PromptStore> PromptSvc<S> {
    pub const fn new(store: S, friendship_svc: Arc<dyn FriendshipManager>) -> Self {
        Self { store, friendship_svc }
    }
}

#[async_trait::async_trait]
impl<S: PromptStore> PromptManager for PromptSvc<S> {
    async fn create_new(
        &self,
        author_id: i32,
        body: &str,
    ) -> Result<PromptWithAuthor, DomainError> {
        match self.store.insert_new(author_id, body).await {
            Err(InsertionError::Technical(e)) => Err(TechnicalError::Database(e).into()),
            Err(InsertionError::UniqueViolation(_)) => Err(ContentError::DuplicatePrompt.into()),
            Ok(prompt_info) => Ok(prompt_info.into()),
        }
    }

    async fn get_for_writing(
        &self,
        requester_id: i32,
        prompt_id: i32,
    ) -> Result<PromptWithAuthor, DomainError> {
        // Prompt must exist
        let prompt = self
            .store
            .get_by_id(prompt_id)
            .await?
            .ok_or(ContentError::NotFound)?;

        // Disallow writing a post in response to one's own prompt
        if requester_id == prompt.author_id {
            return Err(ContentError::OwnPrompt.into());
        }

        // Must be friends to see someone's prompts
        self.friendship_svc
            .are_friends(&UserIdPair::new(requester_id, prompt.author_id)?)
            .await?
            .then_some(prompt.into())
            .ok_or_else(|| ContentError::NotFriends.into())
    }

    async fn single_user_prompts(
        &self,
        user_id: i32,
    ) -> Result<Vec<PromptWithAuthor>, DomainError> {
        self.store
            .single_user_prompts(user_id)
            .await
            .map_err(Into::into)
            .map(vec_into)
    }

    async fn all_friend_prompts(&self, user_id: i32) -> Result<Vec<PromptWithAuthor>, DomainError> {
        self.store
            .all_friend_prompts(user_id)
            .await
            .map_err(Into::into)
            .map(vec_into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::prompt::PromptInfo;
    use crate::{
        domain::{
            content::repository::MockPromptStore, friendship::service::MockFriendshipManager,
        },
        test_utils::dummy_data::make_user,
    };
    use chrono::Utc;
    use mockall::predicate::eq;

    mod create_new {
        use super::*;

        #[tokio::test]
        async fn converts_insertion_errors_into_domain_errors() {
            let mut mock_prompt_repo = MockPromptStore::new();

            let author_id_1 = 442;
            let prompt_body_1 = "any body";

            mock_prompt_repo
                .expect_insert_new()
                .with(eq(author_id_1), eq(prompt_body_1))
                .once()
                .return_once(|_, _| Err(InsertionError::Technical(sqlx::Error::WorkerCrashed)));

            let author_id_2 = 443;
            let prompt_body_2 = "some other body";

            mock_prompt_repo
                .expect_insert_new()
                .with(eq(author_id_2), eq(prompt_body_2))
                .once()
                .return_once(|_, _| {
                    Err(InsertionError::UniqueViolation(String::from(
                        "any uniqueness constraint violation here",
                    )))
                });

            let prompt_svc =
                PromptSvc::new(mock_prompt_repo, Arc::new(MockFriendshipManager::new()));

            let result1 = prompt_svc.create_new(author_id_1, prompt_body_1).await;
            let result2 = prompt_svc.create_new(author_id_2, prompt_body_2).await;

            assert!(matches!(
                result1,
                Err(DomainError::Technical(TechnicalError::Database(
                    sqlx::Error::WorkerCrashed
                )))
            ));
            assert!(matches!(
                result2,
                Err(DomainError::Content(ContentError::DuplicatePrompt))
            ));
        }

        #[tokio::test]
        async fn creates_prompt_with_author_for_successful_insertion() {
            let (user1, user2) = (make_user::number1(), make_user::number2());
            let (user1_id, user2_id) = (user1.id, user2.id);
            let (user1_username, user2_username) = (user1.username.clone(), user2.username.clone());
            let (prompt_body_1, prompt_body_2) = ("Prompt body one!", "Prompt body two?");
            let (prompt_id_1, prompt_id_2) = (354, 355);

            let prompt_info_1 = PromptInfo {
                id: prompt_id_1,
                author_id: user1_id,
                author_username: user1.username.clone(),
                body: prompt_body_1.to_string(),
                created_at: Utc::now(),
            };

            let prompt_info_2 = PromptInfo {
                id: prompt_id_2,
                author_id: user2_id,
                author_username: user2_username.clone(),
                body: prompt_body_2.to_string(),
                created_at: Utc::now(),
            };

            let mut mock_prompt_repo = MockPromptStore::new();
            mock_prompt_repo
                .expect_insert_new()
                .with(eq(user1.id), eq(prompt_body_1))
                .once()
                .return_once(move |_, _| Ok(prompt_info_1));
            mock_prompt_repo
                .expect_insert_new()
                .with(eq(user2.id), eq(prompt_body_2))
                .once()
                .return_once(move |_, _| Ok(prompt_info_2));

            let expected1 = PromptWithAuthor {
                id: prompt_id_1,
                author_username: user1_username,
                body: prompt_body_1.to_string(),
            };
            let expected2 = PromptWithAuthor {
                id: prompt_id_2,
                author_username: user2_username,
                body: prompt_body_2.to_string(),
            };

            let prompt_svc =
                PromptSvc::new(mock_prompt_repo, Arc::new(MockFriendshipManager::new()));

            let actual1 = prompt_svc
                .create_new(user1_id, prompt_body_1)
                .await
                .expect("failed to create prompt 1");
            let actual2 = prompt_svc
                .create_new(user2_id, prompt_body_2)
                .await
                .expect("failed to create prompt 2");

            assert_eq!(actual1, expected1);
            assert_eq!(actual2, expected2);
        }
    }

    // mod get_for_writing {
    //     use super::*;

    //     #[tokio::test]
    //     async fn errors_for_nonexistent_prompt() { todo!() }

    //     #[tokio::test]
    //     async fn disallows_responding_to_ones_own_prompt() { todo!() }

    //     #[tokio::test]
    //     async fn requires_confirmed_friendship_to_see_prompts() { todo!() }

    //     #[tokio::test]
    //     async fn creates_prompt_with_author_for_a_friends_existing_prompt() { todo!() }
    // }
}
