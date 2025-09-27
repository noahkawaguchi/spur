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
        test_utils::{dummy_data, fake_db::FakeUow, mock_repos::MockPostRepo},
    };
    use anyhow::anyhow;

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
                    Ok(Some(dummy_data::post::three_dummies()[i & 1].clone()))
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

    // #[tokio::test]
    // async fn translates_post_insertion_outcomes() {
    //     let author_ids = [11, 22, 33, 44, 55];
    //     let parent_ids = [101, 202, 303, 404, 505];
    //     let post_bodies = ["very", "awesome", "correspondence", "happening", "here"];
    //
    //     let mut mock_post_repo = MockPostRepo::new();
    //     let mut seq = Sequence::new();
    //     mock_post_repo
    //         .expect_insert_new()
    //         .with(eq(author_ids[0]), eq(parent_ids[0]), eq(post_bodies[0]))
    //         .once()
    //         .in_sequence(&mut seq)
    //         .return_once(|_, _, _| Ok(PostInsertionOutcome::Inserted));
    //     mock_post_repo
    //         .expect_insert_new()
    //         .with(eq(author_ids[1]), eq(parent_ids[1]), eq(post_bodies[1]))
    //         .once()
    //         .in_sequence(&mut seq)
    //         .return_once(|_, _, _| Ok(PostInsertionOutcome::ParentMissing));
    //     mock_post_repo
    //         .expect_insert_new()
    //         .with(eq(author_ids[2]), eq(parent_ids[2]), eq(post_bodies[2]))
    //         .once()
    //         .in_sequence(&mut seq)
    //         .return_once(|_, _, _| Ok(PostInsertionOutcome::ParentDeleted));
    //     mock_post_repo
    //         .expect_insert_new()
    //         .with(eq(author_ids[3]), eq(parent_ids[3]), eq(post_bodies[3]))
    //         .once()
    //         .in_sequence(&mut seq)
    //         .return_once(|_, _, _| Ok(PostInsertionOutcome::ParentArchived));
    //     mock_post_repo
    //         .expect_insert_new()
    //         .with(eq(author_ids[4]), eq(parent_ids[4]), eq(post_bodies[4]))
    //         .once()
    //         .in_sequence(&mut seq)
    //         .return_once(|_, _, _| Ok(PostInsertionOutcome::SelfReply));
    //
    //     let post_svc = PostDomainSvc::new(mock_post_repo);
    //     assert!(matches!(
    //         post_svc
    //             .create_new(author_ids[0], parent_ids[0], post_bodies[0])
    //             .await,
    //         Ok(())
    //     ));
    //     assert!(matches!(
    //         post_svc
    //             .create_new(author_ids[1], parent_ids[1], post_bodies[1])
    //             .await,
    //         Err(PostError::NotFound)
    //     ));
    //     assert!(matches!(
    //         post_svc
    //             .create_new(author_ids[2], parent_ids[2], post_bodies[2])
    //             .await,
    //         Err(PostError::DeletedParent)
    //     ));
    //     assert!(matches!(
    //         post_svc
    //             .create_new(author_ids[3], parent_ids[3], post_bodies[3])
    //             .await,
    //         Err(PostError::ArchivedParent)
    //     ));
    //     assert!(matches!(
    //         post_svc
    //             .create_new(author_ids[4], parent_ids[4], post_bodies[4])
    //             .await,
    //         Err(PostError::SelfReply)
    //     ));
    // }
}
