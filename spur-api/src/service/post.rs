use crate::{
    domain::{
        content::{
            error::ContentError,
            repository::PostStore,
            service::{PostManager, PromptManager},
        },
        error::DomainError,
        friendship::{service::FriendshipManager, user_id_pair::UserIdPair},
    },
    repository::insertion_error::InsertionError,
    technical_error::TechnicalError,
    utils::vec_into,
};
use spur_shared::models::PostWithPrompt;
use std::sync::Arc;

pub struct PostSvc<S: PostStore> {
    store: S,
    friendship_svc: Arc<dyn FriendshipManager>,
    prompt_svc: Arc<dyn PromptManager>,
}

impl<S: PostStore> PostSvc<S> {
    pub const fn new(
        store: S,
        friendship_svc: Arc<dyn FriendshipManager>,
        prompt_svc: Arc<dyn PromptManager>,
    ) -> Self {
        Self { store, friendship_svc, prompt_svc }
    }
}

#[async_trait::async_trait]
impl<S: PostStore> PostManager for PostSvc<S> {
    async fn create_new(
        &self,
        author_id: i32,
        prompt_id: i32,
        body: &str,
    ) -> Result<PostWithPrompt, DomainError> {
        // Prompt must exist and be written by a friend to be able to write a post in response
        self.prompt_svc
            .get_for_writing(author_id, prompt_id)
            .await?;

        match self.store.insert_new(author_id, prompt_id, body).await {
            Err(InsertionError::Technical(e)) => Err(TechnicalError::Database(e).into()),
            Err(InsertionError::UniqueViolation(_)) => Err(ContentError::DuplicatePost.into()),
            Ok(post) => Ok(post.into()),
        }
    }

    async fn get_for_reading(
        &self,
        requester_id: i32,
        post_id: i32,
    ) -> Result<PostWithPrompt, DomainError> {
        // Post must exist
        let post = self
            .store
            .get_by_id(post_id)
            .await?
            .ok_or(ContentError::NotFound)?;

        // Only allow reading posts written by oneself or one's friends
        if requester_id == post.author_id
            || self
                .friendship_svc
                .are_friends(&UserIdPair::new(requester_id, post.author_id)?)
                .await?
        {
            Ok(post.into())
        } else {
            Err(ContentError::NotFriends.into())
        }
    }

    async fn single_user_posts(&self, author_id: i32) -> Result<Vec<PostWithPrompt>, DomainError> {
        self.store
            .single_user_posts(author_id)
            .await
            .map_err(Into::into)
            .map(vec_into)
    }

    async fn all_friend_posts(&self, user_id: i32) -> Result<Vec<PostWithPrompt>, DomainError> {
        self.store
            .all_friend_posts(user_id)
            .await
            .map_err(Into::into)
            .map(vec_into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::{
            content::{repository::MockPostStore, service::MockPromptManager},
            friendship::service::MockFriendshipManager,
        },
        models::post::PostInfo,
    };
    use chrono::Utc;
    use mockall::predicate::eq;
    use spur_shared::models::PromptWithAuthor;

    fn make_prompt_with_author() -> PromptWithAuthor {
        PromptWithAuthor {
            id: 444,
            author_username: String::from("any_username"),
            body: String::from("any body here"),
        }
    }

    fn make_post_info() -> PostInfo {
        PostInfo {
            id: 245,
            author_id: 99,
            author_username: String::from("any_username"),
            body: String::from("super cool post body"),
            created_at: Utc::now(),
            edited_at: None,
            prompt_id: 1111,
            prompt_author_username: String::from("any username"),
            prompt_body: String::from("any prompt body"),
        }
    }

    mod create_new {
        use super::*;

        #[tokio::test]
        async fn errors_for_failed_prompt_retrieval() {
            let (author_id, prompt_id) = (284, 9924);

            let mut mock_prompt_svc = MockPromptManager::new();
            mock_prompt_svc
                .expect_get_for_writing()
                .with(eq(author_id), eq(prompt_id))
                .once()
                .return_once(|_, _| Err(ContentError::NotFriends.into()));

            let post_svc = PostSvc::new(
                MockPostStore::new(),
                Arc::new(MockFriendshipManager::new()),
                Arc::new(mock_prompt_svc),
            );

            let result = post_svc
                .create_new(author_id, prompt_id, "cool post body")
                .await;
            assert!(matches!(
                result,
                Err(DomainError::Content(ContentError::NotFriends))
            ));
        }

        #[tokio::test]
        async fn converts_insertion_errors_into_domain_errors() {
            let author_id_1 = 2525;
            let prompt_id_1 = 5;
            let post_body_1 = "super cool post body";

            let author_id_2 = 2443;
            let prompt_id_2 = 999;
            let post_body_2 = "some other really cool post body";

            let mut mock_prompt_svc = MockPromptManager::new();
            mock_prompt_svc
                .expect_get_for_writing()
                .with(eq(author_id_1), eq(prompt_id_1))
                .once()
                .return_once(|_, _| Ok(make_prompt_with_author()));
            mock_prompt_svc
                .expect_get_for_writing()
                .with(eq(author_id_2), eq(prompt_id_2))
                .once()
                .return_once(|_, _| Ok(make_prompt_with_author()));

            let mut mock_post_repo = MockPostStore::new();
            mock_post_repo
                .expect_insert_new()
                .with(eq(author_id_1), eq(prompt_id_1), eq(post_body_1))
                .once()
                .return_once(|_, _, _| Err(InsertionError::Technical(sqlx::Error::PoolClosed)));
            mock_post_repo
                .expect_insert_new()
                .with(eq(author_id_2), eq(prompt_id_2), eq(post_body_2))
                .once()
                .return_once(|_, _, _| {
                    Err(InsertionError::UniqueViolation(String::from(
                        "any uniqueness constraint violation here",
                    )))
                });

            let post_svc = PostSvc::new(
                mock_post_repo,
                Arc::new(MockFriendshipManager::new()),
                Arc::new(mock_prompt_svc),
            );

            let result1 = post_svc
                .create_new(author_id_1, prompt_id_1, post_body_1)
                .await;
            let result2 = post_svc
                .create_new(author_id_2, prompt_id_2, post_body_2)
                .await;

            assert!(matches!(
                result1,
                Err(DomainError::Technical(TechnicalError::Database(
                    sqlx::Error::PoolClosed
                )))
            ));
            assert!(matches!(
                result2,
                Err(DomainError::Content(ContentError::DuplicatePost))
            ));
        }

        #[tokio::test]
        async fn converts_to_post_with_prompt_for_successful_insertion() {
            let post_info = make_post_info();
            let post_info_clone = post_info.clone();
            let post_with_prompt = PostWithPrompt::from(post_info.clone());

            let mut mock_prompt_svc = MockPromptManager::new();
            mock_prompt_svc
                .expect_get_for_writing()
                .with(eq(post_info.author_id), eq(post_info.prompt_id))
                .once()
                .return_once(|_, _| Ok(make_prompt_with_author()));

            let mut mock_post_repo = MockPostStore::new();
            mock_post_repo
                .expect_insert_new()
                .with(
                    eq(post_info.author_id),
                    eq(post_info.prompt_id),
                    eq(post_info.body.clone()),
                )
                .once()
                .return_once(|_, _, _| Ok(post_info_clone));

            let post_svc = PostSvc::new(
                mock_post_repo,
                Arc::new(MockFriendshipManager::new()),
                Arc::new(mock_prompt_svc),
            );

            let result = post_svc
                .create_new(post_info.author_id, post_info.prompt_id, &post_info.body)
                .await
                .expect("returned error despite successful insertion");

            assert_eq!(result, post_with_prompt);
        }
    }

    mod get_for_reading {
        use super::*;

        #[tokio::test]
        async fn errors_for_nonexistent_post() {
            let post_id = 98922;

            let mut mock_post_repo = MockPostStore::new();
            mock_post_repo
                .expect_get_by_id()
                .with(eq(post_id))
                .once()
                .return_once(|_| Ok(None));

            let post_svc = PostSvc::new(
                mock_post_repo,
                Arc::new(MockFriendshipManager::new()),
                Arc::new(MockPromptManager::new()),
            );

            let result = post_svc.get_for_reading(254, post_id).await;

            assert!(matches!(
                result,
                Err(DomainError::Content(ContentError::NotFound))
            ));
        }

        #[tokio::test]
        async fn disallows_post_reading_by_non_friends() {
            let post_info = make_post_info();
            let post_info_clone = post_info.clone();
            let requester_id = post_info.author_id + 966;

            let mut mock_post_repo = MockPostStore::new();
            mock_post_repo
                .expect_get_by_id()
                .with(eq(post_info.id))
                .once()
                .return_once(|_| Ok(Some(post_info_clone)));

            let mut mock_friendship_svc = MockFriendshipManager::new();
            mock_friendship_svc
                .expect_are_friends()
                .with(eq(
                    UserIdPair::new(requester_id, post_info.author_id).unwrap()
                ))
                .once()
                .return_once(|_| Ok(false));

            let post_svc = PostSvc::new(
                mock_post_repo,
                Arc::new(mock_friendship_svc),
                Arc::new(MockPromptManager::new()),
            );

            let result = post_svc.get_for_reading(requester_id, post_info.id).await;
            assert!(matches!(
                result,
                Err(DomainError::Content(ContentError::NotFriends))
            ));
        }

        #[tokio::test]
        async fn converts_to_post_with_prompt_for_ones_own_post() {
            let post_info = make_post_info();
            let post_info_clone = post_info.clone();

            let mut mock_post_repo = MockPostStore::new();
            mock_post_repo
                .expect_get_by_id()
                .with(eq(post_info.id))
                .once()
                .return_once(|_| Ok(Some(post_info_clone)));

            let post_svc = PostSvc::new(
                mock_post_repo,
                Arc::new(MockFriendshipManager::new()),
                Arc::new(MockPromptManager::new()),
            );

            let result = post_svc
                .get_for_reading(post_info.author_id, post_info.id)
                .await
                .expect("failed to get one's own post for reading");

            assert_eq!(result, post_info.into());
        }

        #[tokio::test]
        async fn converts_to_post_with_prompt_for_a_friends_post() {
            let post_info = make_post_info();
            let post_info_clone = post_info.clone();
            let requester_id = post_info.author_id + 61;

            let mut mock_post_repo = MockPostStore::new();
            mock_post_repo
                .expect_get_by_id()
                .with(eq(post_info.id))
                .once()
                .return_once(|_| Ok(Some(post_info_clone)));

            let mut mock_friendship_svc = MockFriendshipManager::new();
            mock_friendship_svc
                .expect_are_friends()
                .with(eq(
                    UserIdPair::new(requester_id, post_info.author_id).unwrap()
                ))
                .once()
                .return_once(|_| Ok(true));

            let post_svc = PostSvc::new(
                mock_post_repo,
                Arc::new(mock_friendship_svc),
                Arc::new(MockPromptManager::new()),
            );

            let result = post_svc
                .get_for_reading(requester_id, post_info.id)
                .await
                .expect("failed to get friend's post for reading");

            assert_eq!(result, post_info.into());
        }
    }

    // Determined that `single_user_posts` and `all_friend_posts` do not need to be tested at this
    // point because they just wrap the repository functions and call `into`.
}
