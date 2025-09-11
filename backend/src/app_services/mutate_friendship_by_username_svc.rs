use crate::{
    app_services::{
        MutateFriendshipByUsername,
        uow::{Tx, UnitOfWork},
    },
    domain::{
        friendship::{
            FriendshipStatus, error::FriendshipError, repository::FriendshipRepo,
            user_id_pair::UserIdPair,
        },
        user::UserRepo,
    },
};
use std::sync::Arc;

pub struct MutateFriendshipByUsernameSvc<U, F> {
    uow: U,
    user_repo: Arc<dyn UserRepo>,
    friendship_repo: F,
}

impl<U, F> MutateFriendshipByUsernameSvc<U, F> {
    pub const fn new(uow: U, user_repo: Arc<dyn UserRepo>, friendship_repo: F) -> Self {
        Self { uow, user_repo, friendship_repo }
    }
}

#[async_trait::async_trait]
impl<U, F> MutateFriendshipByUsername for MutateFriendshipByUsernameSvc<U, F>
where
    U: UnitOfWork,
    F: FriendshipRepo,
{
    async fn add_friend_by_username(
        &self,
        sender_id: i32,
        recipient_username: &str,
    ) -> Result<bool, FriendshipError> {
        // Find the recipient's ID and create an ID pair
        //
        // TODO: this should also be in the transaction
        //
        let recipient_id = self
            .user_repo
            .get_by_username(recipient_username)
            .await?
            .ok_or(FriendshipError::NonexistentUser)?
            .id;
        let ids = UserIdPair::new(sender_id, recipient_id)?;

        let mut tx = self.uow.begin_uow().await?;

        // Determine the pair's current status
        let result = match self.friendship_repo.get_status(tx.exec(), &ids).await? {
            // Already friends, cannot request to become friends
            FriendshipStatus::Friends => Err(FriendshipError::AlreadyFriends),
            // A request from this sender to this recipient already exists, cannot request again
            FriendshipStatus::PendingFrom(id) if id == sender_id => {
                Err(FriendshipError::AlreadyRequested)
            }
            // Already a pending request in the opposite direction, so accept it
            FriendshipStatus::PendingFrom(_) => {
                self.friendship_repo.accept_request(tx.exec(), &ids).await?;
                Ok(true)
            }
            // No existing relationship, create a new request
            FriendshipStatus::Nil => {
                self.friendship_repo
                    .new_request(tx.exec(), &ids, sender_id)
                    .await?;
                Ok(false)
            }
        };

        tx.commit_uow().await?;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        repository::error::RepoError,
        test_utils::{dummy_data, fake_db::FakeUow},
    };
    use mockall::predicate::eq;
    use sqlx::PgExecutor;

    #[allow(clippy::type_complexity)]
    #[derive(Default)]
    struct MockFriendshipRepo {
        new_request: Option<Box<dyn Fn(&UserIdPair, i32) -> Result<(), RepoError> + Send + Sync>>,
        accept_request: Option<Box<dyn Fn(&UserIdPair) -> Result<(), RepoError> + Send + Sync>>,
        get_status:
            Option<Box<dyn Fn(&UserIdPair) -> Result<FriendshipStatus, RepoError> + Send + Sync>>,
    }

    #[async_trait::async_trait]
    impl FriendshipRepo for MockFriendshipRepo {
        async fn new_request(
            &self,
            _exec: impl PgExecutor<'_>,
            ids: &UserIdPair,
            requester_id: i32,
        ) -> Result<(), RepoError> {
            (self.new_request.as_ref().unwrap())(ids, requester_id)
        }

        async fn accept_request(
            &self,
            _exec: impl PgExecutor<'_>,
            ids: &UserIdPair,
        ) -> Result<(), RepoError> {
            (self.accept_request.as_ref().unwrap())(ids)
        }

        async fn get_status(
            &self,
            _exec: impl PgExecutor<'_>,
            ids: &UserIdPair,
        ) -> Result<FriendshipStatus, RepoError> {
            (self.get_status.as_ref().unwrap())(ids)
        }
    }

    mod add_friend {
        use super::*;
        use crate::domain::user::MockUserRepo;

        #[tokio::test]
        async fn disallows_sending_a_friend_request_to_a_friend() {
            let my_friend = dummy_data::user::number1();
            let my_friend_clone = my_friend.clone();
            let my_id = my_friend.id - 1;
            let ids = UserIdPair::new(my_id, my_friend.id).unwrap();

            let mut mock_user_repo = MockUserRepo::new();
            mock_user_repo
                .expect_get_by_username()
                .with(eq(my_friend.username.clone()))
                .once()
                .return_once(|_| Ok(Some(my_friend_clone)));

            let mock_friendship_repo = MockFriendshipRepo {
                get_status: Some(Box::new(move |&passed_ids| {
                    assert_eq!(ids, passed_ids);
                    Ok(FriendshipStatus::Friends)
                })),
                ..Default::default()
            };

            let (fake_uow, probe) = FakeUow::with_probe();

            let friendship_svc = MutateFriendshipByUsernameSvc::new(
                fake_uow,
                Arc::new(mock_user_repo),
                mock_friendship_repo,
            );
            let result = friendship_svc
                .add_friend_by_username(my_id, &my_friend.username)
                .await;

            assert!(matches!(result, Err(FriendshipError::AlreadyFriends)));
            assert!(probe.commit_called());
        }

        #[tokio::test]
        async fn disallows_duplicate_friend_requests() {
            let desired_friend = dummy_data::user::number2();
            let desired_friend_clone = desired_friend.clone();
            let my_id = desired_friend.id + 3;
            let ids = UserIdPair::new(my_id, desired_friend.id).unwrap();

            let mut mock_user_repo = MockUserRepo::new();
            mock_user_repo
                .expect_get_by_username()
                .with(eq(desired_friend.username.clone()))
                .once()
                .return_once(|_| Ok(Some(desired_friend_clone)));

            let mock_friendship_repo = MockFriendshipRepo {
                get_status: Some(Box::new(move |&passed_ids| {
                    assert_eq!(passed_ids, ids);
                    Ok(FriendshipStatus::PendingFrom(my_id))
                })),
                ..Default::default()
            };

            let (fake_uow, probe) = FakeUow::with_probe();

            let friendship_svc = MutateFriendshipByUsernameSvc::new(
                fake_uow,
                Arc::new(mock_user_repo),
                mock_friendship_repo,
            );
            let result = friendship_svc
                .add_friend_by_username(my_id, &desired_friend.username)
                .await;

            assert!(matches!(result, Err(FriendshipError::AlreadyRequested)));
            assert!(probe.commit_called());
        }

        #[tokio::test]
        async fn accepts_a_friend_request_in_the_opposite_direction() {
            let added_me = dummy_data::user::number3();
            let added_me_clone = added_me.clone();
            let my_id = added_me.id + 100;
            let ids = UserIdPair::new(my_id, added_me.id).unwrap();

            let mut mock_user_repo = MockUserRepo::new();
            mock_user_repo
                .expect_get_by_username()
                .with(eq(added_me.username.clone()))
                .once()
                .return_once(|_| Ok(Some(added_me_clone)));

            let mock_friendship_repo = MockFriendshipRepo {
                get_status: Some(Box::new(move |&passed_ids| {
                    assert_eq!(ids, passed_ids);
                    Ok(FriendshipStatus::PendingFrom(added_me.id))
                })),
                accept_request: Some(Box::new(move |&passed_ids| {
                    assert_eq!(ids, passed_ids);
                    Ok(())
                })),
                ..Default::default()
            };

            let (fake_uow, probe) = FakeUow::with_probe();

            let friendship_svc = MutateFriendshipByUsernameSvc::new(
                fake_uow,
                Arc::new(mock_user_repo),
                mock_friendship_repo,
            );
            let result = friendship_svc
                .add_friend_by_username(my_id, &added_me.username)
                .await;

            assert!(matches!(result, Ok(true)));
            assert!(probe.commit_called());
        }

        #[tokio::test]
        async fn creates_a_request_if_no_relationship() {
            let does_not_know_me = dummy_data::user::number4();
            let does_not_know_me_clone = does_not_know_me.clone();
            let my_id = does_not_know_me.id - 7;
            let ids = UserIdPair::new(my_id, does_not_know_me.id).unwrap();

            let mut mock_user_svc = MockUserRepo::new();
            mock_user_svc
                .expect_get_by_username()
                .with(eq(does_not_know_me.username.clone()))
                .once()
                .return_once(|_| Ok(Some(does_not_know_me_clone)));

            let mock_friendship_repo = MockFriendshipRepo {
                get_status: Some(Box::new(move |&passed_ids| {
                    assert_eq!(ids, passed_ids);
                    Ok(FriendshipStatus::Nil)
                })),
                new_request: Some(Box::new(move |&passed_ids, passed_my_id| {
                    assert_eq!(ids, passed_ids);
                    assert_eq!(my_id, passed_my_id);
                    Ok(())
                })),
                ..Default::default()
            };

            let (fake_uow, probe) = FakeUow::with_probe();

            let friendship_svc = MutateFriendshipByUsernameSvc::new(
                fake_uow,
                Arc::new(mock_user_svc),
                mock_friendship_repo,
            );
            let result = friendship_svc
                .add_friend_by_username(my_id, &does_not_know_me.username)
                .await;

            assert!(matches!(result, Ok(false)));
            assert!(probe.commit_called());
        }
    }
}
