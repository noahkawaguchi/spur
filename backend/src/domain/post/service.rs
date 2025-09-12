use crate::domain::post::{PostError, PostManager, PostRepo};

pub struct PostSvc<S: PostRepo> {
    store: S,
}

impl<S: PostRepo> PostSvc<S> {
    pub const fn new(store: S) -> Self { Self { store } }
}

#[async_trait::async_trait]
impl<S: PostRepo> PostManager for PostSvc<S> {
    async fn create_new(
        &self,
        author_id: i32,
        parent_id: i32,
        body: &str,
    ) -> Result<(), PostError> {
        self.store
            .insert_new(author_id, parent_id, body)
            .await
            .map_err(Into::into)
            .and_then(TryFrom::try_from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        RepoError,
        post::{MockPostRepo, PostInsertionOutcome},
    };
    use anyhow::anyhow;
    use mockall::{Sequence, predicate::eq};

    #[tokio::test]
    async fn translates_repo_errors() {
        let author_ids = [1, 2, 3, 4];
        let parent_ids = [10, 20, 30, 40];
        let post_bodies = ["super", "cool", "post", "bodies"];

        let mut mock_post_repo = MockPostRepo::new();
        let mut seq = Sequence::new();
        mock_post_repo
            .expect_insert_new()
            .with(eq(author_ids[0]), eq(parent_ids[0]), eq(post_bodies[0]))
            .once()
            .in_sequence(&mut seq)
            .return_once(|_, _, _| {
                Err(RepoError::UniqueViolation(String::from(
                    "post_author_parent_unique",
                )))
            });
        mock_post_repo
            .expect_insert_new()
            .with(eq(author_ids[1]), eq(parent_ids[1]), eq(post_bodies[1]))
            .once()
            .in_sequence(&mut seq)
            .return_once(|_, _, _| {
                Err(RepoError::UniqueViolation(String::from(
                    "some unique constraint violation here",
                )))
            });
        mock_post_repo
            .expect_insert_new()
            .with(eq(author_ids[3]), eq(parent_ids[3]), eq(post_bodies[3]))
            .once()
            .in_sequence(&mut seq)
            .return_once(|_, _, _| Err(RepoError::Technical(anyhow!("something went wrong!"))));

        let post_svc = PostSvc::new(mock_post_repo);
        assert!(matches!(
            post_svc
                .create_new(author_ids[0], parent_ids[0], post_bodies[0])
                .await,
            Err(PostError::DuplicateReply)
        ));
        assert!(matches!(
            post_svc
                .create_new(author_ids[1], parent_ids[1], post_bodies[1])
                .await,
            Err(PostError::Internal(e)) if e.to_string() ==
            "Unexpected unique violation: some unique constraint violation here"
        ));
        assert!(matches!(
            post_svc.create_new(author_ids[3], parent_ids[3], post_bodies[3]).await,
            Err(PostError::Internal(e)) if e .to_string() == "something went wrong!"
        ));
    }

    #[tokio::test]
    async fn translates_post_insertion_outcomes() {
        let author_ids = [11, 22, 33, 44, 55];
        let parent_ids = [101, 202, 303, 404, 505];
        let post_bodies = ["very", "awesome", "correspondence", "happening", "here"];

        let mut mock_post_repo = MockPostRepo::new();
        let mut seq = Sequence::new();
        mock_post_repo
            .expect_insert_new()
            .with(eq(author_ids[0]), eq(parent_ids[0]), eq(post_bodies[0]))
            .once()
            .in_sequence(&mut seq)
            .return_once(|_, _, _| Ok(PostInsertionOutcome::Inserted));
        mock_post_repo
            .expect_insert_new()
            .with(eq(author_ids[1]), eq(parent_ids[1]), eq(post_bodies[1]))
            .once()
            .in_sequence(&mut seq)
            .return_once(|_, _, _| Ok(PostInsertionOutcome::ParentMissing));
        mock_post_repo
            .expect_insert_new()
            .with(eq(author_ids[2]), eq(parent_ids[2]), eq(post_bodies[2]))
            .once()
            .in_sequence(&mut seq)
            .return_once(|_, _, _| Ok(PostInsertionOutcome::ParentDeleted));
        mock_post_repo
            .expect_insert_new()
            .with(eq(author_ids[3]), eq(parent_ids[3]), eq(post_bodies[3]))
            .once()
            .in_sequence(&mut seq)
            .return_once(|_, _, _| Ok(PostInsertionOutcome::ParentArchived));
        mock_post_repo
            .expect_insert_new()
            .with(eq(author_ids[4]), eq(parent_ids[4]), eq(post_bodies[4]))
            .once()
            .in_sequence(&mut seq)
            .return_once(|_, _, _| Ok(PostInsertionOutcome::SelfReply));

        let post_svc = PostSvc::new(mock_post_repo);
        assert!(matches!(
            post_svc
                .create_new(author_ids[0], parent_ids[0], post_bodies[0])
                .await,
            Ok(())
        ));
        assert!(matches!(
            post_svc
                .create_new(author_ids[1], parent_ids[1], post_bodies[1])
                .await,
            Err(PostError::NotFound)
        ));
        assert!(matches!(
            post_svc
                .create_new(author_ids[2], parent_ids[2], post_bodies[2])
                .await,
            Err(PostError::DeletedParent)
        ));
        assert!(matches!(
            post_svc
                .create_new(author_ids[3], parent_ids[3], post_bodies[3])
                .await,
            Err(PostError::ArchivedParent)
        ));
        assert!(matches!(
            post_svc
                .create_new(author_ids[4], parent_ids[4], post_bodies[4])
                .await,
            Err(PostError::SelfReply)
        ));
    }
}
