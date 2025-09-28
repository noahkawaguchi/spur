use crate::{
    app_services::uow::{Tx, UnitOfWork},
    domain::post::{PostError, PostRepo, PostSvc},
};

pub struct PostDomainSvc<U, R> {
    uow: U,
    repo: R,
}

impl<U, R> PostDomainSvc<U, R> {
    pub const fn new(uow: U, repo: R) -> Self { Self { uow, repo } }
}

#[async_trait::async_trait]
impl<U, R> PostSvc for PostDomainSvc<U, R>
where
    U: UnitOfWork,
    R: PostRepo,
{
    async fn create_new(
        &self,
        author_id: i32,
        parent_id: i32,
        body: &str,
    ) -> Result<(), PostError> {
        // Disallow writing posts in response to nonexistent, deleted, archived, or one's own posts

        let mut tx = self.uow.begin_uow().await?;

        let parent = self
            .repo
            .get_by_id(tx.exec(), parent_id)
            .await?
            .ok_or(PostError::NotFound)?;

        if parent.deleted_at.is_some() {
            return Err(PostError::DeletedParent);
        }
        if parent.archived_at.is_some() {
            return Err(PostError::ArchivedParent);
        }
        if parent
            .author_id
            .is_some_and(|parent_author_id| parent_author_id == author_id)
        {
            return Err(PostError::SelfReply);
        }

        self.repo
            .insert_new(tx.exec(), author_id, parent_id, body)
            .await?;

        tx.commit_uow().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::RepoError,
        models::post::Post,
        test_utils::{dummy_data, fake_db::FakeUow, mock_repos::MockPostRepo},
    };
    use anyhow::anyhow;
    use chrono::Utc;

    async fn run_unacceptable_parent_test(
        author_id: i32,
        parent_post: Option<Post>,
        expected_post_error: PostError,
    ) {
        let parent_post_id = parent_post.as_ref().map_or(543, |post| post.id);

        let mock_repo = MockPostRepo {
            get_by_id: Some(Box::new(move |passed_id| {
                assert_eq!(parent_post_id, passed_id);
                Ok(parent_post.clone())
            })),
            ..Default::default()
        };

        let (fake_uow, probe) = FakeUow::with_probe();
        let result = PostDomainSvc::new(fake_uow, mock_repo)
            .create_new(author_id, parent_post_id, "My parent is unacceptable")
            .await;

        assert!(matches!(result, Err(e) if e == expected_post_error));
        assert!(!probe.commit_called());
    }

    #[tokio::test]
    async fn disallows_replying_to_a_nonexistent_post() {
        run_unacceptable_parent_test(20, None, PostError::NotFound).await;
    }

    #[tokio::test]
    async fn disallows_replying_to_a_deleted_post() {
        let mut deleted_parent = dummy_data::post::number1();
        deleted_parent.deleted_at = Some(Utc::now());
        run_unacceptable_parent_test(
            deleted_parent.author_id.unwrap_or(41) + 1,
            Some(deleted_parent),
            PostError::DeletedParent,
        )
        .await;
    }

    #[tokio::test]
    async fn disallows_replying_to_an_archived_post() {
        let mut archived_parent = dummy_data::post::number1();
        archived_parent.archived_at = Some(Utc::now());
        run_unacceptable_parent_test(
            archived_parent.author_id.unwrap_or(43) - 1,
            Some(archived_parent),
            PostError::ArchivedParent,
        )
        .await;
    }

    #[tokio::test]
    async fn disallows_replying_to_ones_own_post() {
        let self_reply_parent = dummy_data::post::number1();
        run_unacceptable_parent_test(
            self_reply_parent.author_id.unwrap(),
            Some(self_reply_parent),
            PostError::SelfReply,
        )
        .await;
    }

    #[tokio::test]
    async fn creates_post_and_commits_if_all_conditions_are_met() {
        let parent_post = dummy_data::post::number1();
        let parent_post_id = parent_post.id;
        let new_post_author_id = parent_post.author_id.unwrap() + 15;
        let new_post_body = "This is a new post that should work";

        let mock_repo = MockPostRepo {
            get_by_id: Some(Box::new(move |passed_id| {
                assert_eq!(parent_post_id, passed_id);
                Ok(Some(parent_post.clone()))
            })),
            insert_new: Some(Box::new(
                move |passed_author_id, passed_parent_id, passed_post_body| {
                    assert_eq!(new_post_author_id, passed_author_id);
                    assert_eq!(parent_post_id, passed_parent_id);
                    assert_eq!(new_post_body, passed_post_body);
                    Ok(())
                },
            )),
        };

        let (fake_uow, probe) = FakeUow::with_probe();
        let result = PostDomainSvc::new(fake_uow, mock_repo)
            .create_new(new_post_author_id, parent_post_id, new_post_body)
            .await;

        assert!(matches!(result, Ok(())));
        assert!(probe.commit_called());
    }

    #[tokio::test]
    async fn translates_repo_errors() {
        let author_ids = [1, 2, 3, 4, 5];
        let parent_ids = [10, 20, 30, 40, 50];
        let post_bodies = ["super", "cool", "post", "bodies", "here"];

        for (i, (repo_error, post_error)) in [
            (
                RepoError::UniqueViolation(String::from("post_author_parent_unique")),
                PostError::DuplicateReply,
            ),
            (
                RepoError::UniqueViolation(String::from("some unique constraint violation here")),
                PostError::Internal(anyhow!(
                    "Unexpected unique violation: some unique constraint violation here"
                )),
            ),
            (
                RepoError::CheckViolation(String::from("text_non_empty")),
                PostError::Internal(anyhow!(
                    "Empty field made it past request validation: text_non_empty",
                )),
            ),
            (
                RepoError::CheckViolation(String::from("some check violation")),
                PostError::Internal(anyhow!("Unexpected check violation: some check violation")),
            ),
            (
                RepoError::Technical(anyhow!("something went wrong!")),
                PostError::Internal(anyhow!("something went wrong!")),
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let mock_post_repo = MockPostRepo {
                get_by_id: Some(Box::new(move |passed_id| {
                    assert_eq!(parent_ids[i], passed_id);
                    // Alternating between the first and second dummy posts because the third is
                    // marked as deleted, which correctly causes a different error
                    Ok(Some(if i & 1 == 1 {
                        dummy_data::post::number1()
                    } else {
                        dummy_data::post::number2()
                    }))
                })),
                insert_new: Some(Box::new(
                    move |passed_author_id, passed_parent_id, passed_post_body| {
                        assert_eq!(author_ids[i], passed_author_id);
                        assert_eq!(parent_ids[i], passed_parent_id);
                        assert_eq!(post_bodies[i], passed_post_body);
                        Err(repo_error.clone())
                    },
                )),
            };

            let (fake_uow, probe) = FakeUow::with_probe();
            let result = PostDomainSvc::new(fake_uow, mock_post_repo)
                .create_new(author_ids[i], parent_ids[i], post_bodies[i])
                .await;

            assert!(matches!(result, Err(e) if e == post_error));
            assert!(!probe.commit_called());
        }
    }
}
