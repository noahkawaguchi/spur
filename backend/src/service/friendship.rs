use crate::{
    domain::{
        friendship::{
            FriendshipStatus, error::FriendshipError, repository::FriendshipStore,
            service::FriendshipManager, user_id_pair::UserIdPair,
        },
        user::UserManager,
    },
    repository::uow::{Tx, UnitOfWork},
};
use std::sync::Arc;

pub struct FriendshipSvc<U, S> {
    uow: U,
    store: S,
    user_svc: Arc<dyn UserManager>,
}

impl<U, S> FriendshipSvc<U, S> {
    pub const fn new(uow: U, store: S, user_svc: Arc<dyn UserManager>) -> Self {
        Self { uow, store, user_svc }
    }
}

#[async_trait::async_trait]
impl<U, S> FriendshipManager for FriendshipSvc<U, S>
where
    U: UnitOfWork,
    S: FriendshipStore,
{
    async fn add_friend(
        &self,
        sender_id: i32,
        recipient_username: &str,
    ) -> Result<bool, FriendshipError> {
        // Find the recipient's ID and create an ID pair
        //
        // TODO: this should also be in the transaction
        //
        let recipient_id = self.user_svc.get_by_username(recipient_username).await?.id;
        let ids = UserIdPair::new(sender_id, recipient_id)?;

        let mut tx = self.uow.begin_uow().await?;

        // Determine the pair's current status
        let result = match self.store.get_status(tx.exec(), &ids).await? {
            // Already friends, cannot request to become friends
            FriendshipStatus::Friends => Err(FriendshipError::AlreadyFriends),
            // A request from this sender to this recipient already exists, cannot request again
            FriendshipStatus::PendingFrom(id) if id == sender_id => {
                Err(FriendshipError::AlreadyRequested)
            }
            // Already a pending request in the opposite direction, so accept it
            FriendshipStatus::PendingFrom(_) => {
                self.store.accept_request(tx.exec(), &ids).await?;
                Ok(true)
            }
            // No existing relationship, create a new request
            FriendshipStatus::Nil => {
                self.store.new_request(tx.exec(), &ids, sender_id).await?;
                Ok(false)
            }
        };

        tx.commit_uow().await?;
        result
    }

    async fn get_friends(&self, id: i32) -> Result<Vec<String>, FriendshipError> {
        self.store
            .get_friends(self.uow.single_exec(), id)
            .await
            .map_err(Into::into)
    }

    async fn get_requests(&self, id: i32) -> Result<Vec<String>, FriendshipError> {
        self.store
            .get_requests(self.uow.single_exec(), id)
            .await
            .map_err(Into::into)
    }

    async fn are_friends(&self, ids: &UserIdPair) -> Result<bool, FriendshipError> {
        Ok(self.store.get_status(self.uow.single_exec(), ids).await? == FriendshipStatus::Friends)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::user::MockUserManager,
        repository::error::RepoError,
        test_utils::{
            dummy_data,
            fake_db::{FakeUow, fake_pool},
        },
    };
    use mockall::predicate::eq;
    use sqlx::PgExecutor;

    #[allow(clippy::type_complexity)]
    #[derive(Default)]
    struct MockFriendshipStore {
        new_request: Option<Box<dyn Fn(&UserIdPair, i32) -> Result<(), RepoError> + Send + Sync>>,
        accept_request: Option<Box<dyn Fn(&UserIdPair) -> Result<(), RepoError> + Send + Sync>>,
        get_status:
            Option<Box<dyn Fn(&UserIdPair) -> Result<FriendshipStatus, RepoError> + Send + Sync>>,
        get_friends: Option<Box<dyn Fn(i32) -> Result<Vec<String>, RepoError> + Send + Sync>>,
        get_requests: Option<Box<dyn Fn(i32) -> Result<Vec<String>, RepoError> + Send + Sync>>,
    }

    #[async_trait::async_trait]
    impl FriendshipStore for MockFriendshipStore {
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

        async fn get_friends(
            &self,
            _exec: impl PgExecutor<'_>,
            id: i32,
        ) -> Result<Vec<String>, RepoError> {
            (self.get_friends.as_ref().unwrap())(id)
        }

        async fn get_requests(
            &self,
            _exec: impl PgExecutor<'_>,
            id: i32,
        ) -> Result<Vec<String>, RepoError> {
            (self.get_requests.as_ref().unwrap())(id)
        }
    }

    mod add_friend {
        use super::*;

        #[tokio::test]
        async fn disallows_sending_a_friend_request_to_a_friend() {
            let my_friend = dummy_data::user::number1();
            let my_friend_clone = my_friend.clone();
            let my_id = my_friend.id - 1;
            let ids = UserIdPair::new(my_id, my_friend.id).unwrap();

            let mut mock_user_svc = MockUserManager::new();
            mock_user_svc
                .expect_get_by_username()
                .with(eq(my_friend.username.clone()))
                .once()
                .return_once(|_| Ok(my_friend_clone));

            let mock_friendship_repo = MockFriendshipStore {
                get_status: Some(Box::new(move |&passed_ids| {
                    assert_eq!(ids, passed_ids);
                    Ok(FriendshipStatus::Friends)
                })),
                ..Default::default()
            };

            let (fake_uow, probe) = FakeUow::with_probe();

            let friendship_svc =
                FriendshipSvc::new(fake_uow, mock_friendship_repo, Arc::new(mock_user_svc));
            let result = friendship_svc.add_friend(my_id, &my_friend.username).await;

            assert!(matches!(result, Err(FriendshipError::AlreadyFriends)));
            assert!(probe.commit_called());
        }

        #[tokio::test]
        async fn disallows_duplicate_friend_requests() {
            let desired_friend = dummy_data::user::number2();
            let desired_friend_clone = desired_friend.clone();
            let my_id = desired_friend.id + 3;
            let ids = UserIdPair::new(my_id, desired_friend.id).unwrap();

            let mut mock_user_svc = MockUserManager::new();
            mock_user_svc
                .expect_get_by_username()
                .with(eq(desired_friend.username.clone()))
                .once()
                .return_once(|_| Ok(desired_friend_clone));

            let mock_friendship_repo = MockFriendshipStore {
                get_status: Some(Box::new(move |&passed_ids| {
                    assert_eq!(passed_ids, ids);
                    Ok(FriendshipStatus::PendingFrom(my_id))
                })),
                ..Default::default()
            };

            let (fake_uow, probe) = FakeUow::with_probe();

            let friendship_svc =
                FriendshipSvc::new(fake_uow, mock_friendship_repo, Arc::new(mock_user_svc));
            let result = friendship_svc
                .add_friend(my_id, &desired_friend.username)
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

            let mut mock_user_svc = MockUserManager::new();
            mock_user_svc
                .expect_get_by_username()
                .with(eq(added_me.username.clone()))
                .once()
                .return_once(|_| Ok(added_me_clone));

            let mock_friendship_repo = MockFriendshipStore {
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

            let friendship_svc =
                FriendshipSvc::new(fake_uow, mock_friendship_repo, Arc::new(mock_user_svc));
            let result = friendship_svc.add_friend(my_id, &added_me.username).await;

            assert!(matches!(result, Ok(true)));
            assert!(probe.commit_called());
        }

        #[tokio::test]
        async fn creates_a_request_if_no_relationship() {
            let does_not_know_me = dummy_data::user::number4();
            let does_not_know_me_clone = does_not_know_me.clone();
            let my_id = does_not_know_me.id - 7;
            let ids = UserIdPair::new(my_id, does_not_know_me.id).unwrap();

            let mut mock_user_svc = MockUserManager::new();
            mock_user_svc
                .expect_get_by_username()
                .with(eq(does_not_know_me.username.clone()))
                .once()
                .return_once(|_| Ok(does_not_know_me_clone));

            let mock_friendship_repo = MockFriendshipStore {
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

            let friendship_svc =
                FriendshipSvc::new(fake_uow, mock_friendship_repo, Arc::new(mock_user_svc));
            let result = friendship_svc
                .add_friend(my_id, &does_not_know_me.username)
                .await;

            assert!(matches!(result, Ok(false)));
            assert!(probe.commit_called());
        }
    }

    #[tokio::test]
    async fn correctly_reports_friendship_as_bool() {
        let ids1 = UserIdPair::new(15, 2).unwrap();
        let ids2 = UserIdPair::new(999, 55).unwrap();
        let ids3 = UserIdPair::new(739, 24252).unwrap();
        let ids4 = UserIdPair::new(9, 10).unwrap();

        let ids2_lesser = ids2.lesser();
        let ids3_greater = ids3.greater();

        let mock_friendship_repo = MockFriendshipStore {
            get_status: Some(Box::new(move |&passed_ids| {
                Ok(match passed_ids {
                    ids if ids == ids1 => FriendshipStatus::Friends,
                    ids if ids == ids2 => FriendshipStatus::PendingFrom(ids2_lesser),
                    ids if ids == ids3 => FriendshipStatus::PendingFrom(ids3_greater),
                    ids if ids == ids4 => FriendshipStatus::Nil,
                    _ => panic!("unexpected IDs passed"),
                })
            })),
            ..Default::default()
        };

        let friendship_svc = FriendshipSvc::new(
            fake_pool(),
            mock_friendship_repo,
            Arc::new(MockUserManager::new()),
        );

        assert!(friendship_svc.are_friends(&ids1).await.unwrap());
        assert!(!friendship_svc.are_friends(&ids2).await.unwrap());
        assert!(!friendship_svc.are_friends(&ids3).await.unwrap());
        assert!(!friendship_svc.are_friends(&ids4).await.unwrap());
    }

    // Determined that testing `get_friends` and `get_requests` would be trivial now that the
    // repository returns the usernames directly
}
