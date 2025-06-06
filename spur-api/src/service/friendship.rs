use crate::domain::{
    error::DomainError,
    friendship::{
        FriendshipStatus, error::FriendshipError, repository::FriendshipStore,
        service::FriendshipManager, user_id_pair::UserIdPair,
    },
    user::UserManager,
};
use std::sync::Arc;

pub struct FriendshipSvc<S: FriendshipStore> {
    store: S,
    user_svc: Arc<dyn UserManager>,
}

impl<S: FriendshipStore> FriendshipSvc<S> {
    pub const fn new(store: S, user_svc: Arc<dyn UserManager>) -> Self { Self { store, user_svc } }
}

#[async_trait::async_trait]
impl<S: FriendshipStore> FriendshipManager for FriendshipSvc<S> {
    async fn add_friend(
        &self,
        sender_id: i32,
        recipient_username: &str,
    ) -> Result<bool, DomainError> {
        // Find the recipient's ID and create an ID pair
        let recipient_id = self.user_svc.get_by_username(recipient_username).await?.id;
        let ids = UserIdPair::new(sender_id, recipient_id)?;

        // Determine the pair's current status
        match self.store.get_status(&ids).await? {
            // Already friends, cannot request to become friends
            FriendshipStatus::Friends => Err(FriendshipError::AlreadyFriends.into()),

            // A request from this sender to this recipient already exists, cannot request again
            FriendshipStatus::PendingFrom(id) if id == sender_id => {
                Err(FriendshipError::AlreadyRequested.into())
            }

            // Already a pending request in the opposite direction, so accept it
            FriendshipStatus::PendingFrom(_) => {
                self.store.accept_request(&ids).await?;
                Ok(true)
            }

            // No existing relationship, create a new request
            FriendshipStatus::Nil => {
                self.store.new_request(&ids, sender_id).await?;
                Ok(false)
            }
        }
    }

    async fn get_friends(&self, id: i32) -> Result<Vec<String>, DomainError> {
        futures::future::try_join_all(
            self.store
                .get_friends(id)
                .await?
                .into_iter()
                .map(|id| async move { Ok(self.user_svc.get_by_id(id).await?.username) }),
        )
        .await
    }

    async fn get_requests(&self, id: i32) -> Result<Vec<String>, DomainError> {
        futures::future::try_join_all(
            self.store
                .get_requests(id)
                .await?
                .into_iter()
                .map(|id| async move { Ok(self.user_svc.get_by_id(id).await?.username) }),
        )
        .await
    }

    async fn are_friends(&self, ids: &UserIdPair) -> Result<bool, DomainError> {
        Ok(self.store.get_status(ids).await? == FriendshipStatus::Friends)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::{friendship::repository::MockFriendshipStore, user::MockUserManager},
        test_utils::dummy_data::make_user,
    };
    use mockall::predicate::eq;

    mod add_friend {
        use crate::test_utils::dummy_data::make_user;

        use super::*;

        #[tokio::test]
        async fn disallows_sending_a_friend_request_to_a_friend() {
            let my_friend = make_user::number1();
            let my_friend_clone = my_friend.clone();
            let my_id = my_friend.id - 1;
            let ids = UserIdPair::new(my_id, my_friend.id).unwrap();

            let mut mock_user_svc = MockUserManager::new();
            mock_user_svc
                .expect_get_by_username()
                .with(eq(my_friend.username.clone()))
                .once()
                .return_once(|_| Ok(my_friend_clone));

            let mut mock_friendship_repo = MockFriendshipStore::new();
            mock_friendship_repo
                .expect_get_status()
                .with(eq(ids))
                .once()
                .return_once(|_| Ok(FriendshipStatus::Friends));

            let friendship_svc = FriendshipSvc::new(mock_friendship_repo, Arc::new(mock_user_svc));
            let result = friendship_svc.add_friend(my_id, &my_friend.username).await;

            assert!(matches!(
                result,
                Err(DomainError::Friendship(FriendshipError::AlreadyFriends)),
            ));
        }

        #[tokio::test]
        async fn disallows_duplicate_friend_requests() {
            let desired_friend = make_user::number2();
            let desired_friend_clone = desired_friend.clone();
            let my_id = desired_friend.id + 3;
            let ids = UserIdPair::new(my_id, desired_friend.id).unwrap();

            let mut mock_user_svc = MockUserManager::new();
            mock_user_svc
                .expect_get_by_username()
                .with(eq(desired_friend.username.clone()))
                .once()
                .return_once(|_| Ok(desired_friend_clone));

            let mut mock_friendship_repo = MockFriendshipStore::new();
            mock_friendship_repo
                .expect_get_status()
                .with(eq(ids))
                .once()
                .return_once(move |_| Ok(FriendshipStatus::PendingFrom(my_id)));

            let friendship_svc = FriendshipSvc::new(mock_friendship_repo, Arc::new(mock_user_svc));
            let result = friendship_svc
                .add_friend(my_id, &desired_friend.username)
                .await;

            assert!(matches!(
                result,
                Err(DomainError::Friendship(FriendshipError::AlreadyRequested)),
            ));
        }

        #[tokio::test]
        async fn accepts_a_friend_request_in_the_opposite_direction() {
            let added_me = make_user::number3();
            let added_me_clone = added_me.clone();
            let my_id = added_me.id + 100;
            let ids = UserIdPair::new(my_id, added_me.id).unwrap();

            let mut mock_user_svc = MockUserManager::new();
            mock_user_svc
                .expect_get_by_username()
                .with(eq(added_me.username.clone()))
                .once()
                .return_once(|_| Ok(added_me_clone));

            let mut mock_friendship_repo = MockFriendshipStore::new();
            mock_friendship_repo
                .expect_get_status()
                .with(eq(ids.clone()))
                .once()
                .return_once(move |_| Ok(FriendshipStatus::PendingFrom(added_me.id)));
            mock_friendship_repo
                .expect_accept_request()
                .with(eq(ids))
                .once()
                .return_once(|_| Ok(()));

            let friendship_svc = FriendshipSvc::new(mock_friendship_repo, Arc::new(mock_user_svc));
            let result = friendship_svc.add_friend(my_id, &added_me.username).await;

            assert!(matches!(result, Ok(true)));
        }

        #[tokio::test]
        async fn creates_a_request_if_no_relationship() {
            let does_not_know_me = make_user::number4();
            let does_not_know_me_clone = does_not_know_me.clone();
            let my_id = does_not_know_me.id - 7;
            let ids = UserIdPair::new(my_id, does_not_know_me.id).unwrap();

            let mut mock_user_svc = MockUserManager::new();
            mock_user_svc
                .expect_get_by_username()
                .with(eq(does_not_know_me.username.clone()))
                .once()
                .return_once(|_| Ok(does_not_know_me_clone));

            let mut mock_friendship_repo = MockFriendshipStore::new();
            mock_friendship_repo
                .expect_get_status()
                .with(eq(ids.clone()))
                .once()
                .return_once(move |_| Ok(FriendshipStatus::Nil));
            mock_friendship_repo.expect_accept_request().never();
            mock_friendship_repo
                .expect_new_request()
                .with(eq(ids), eq(my_id))
                .once()
                .return_once(|_, _| Ok(()));

            let friendship_svc = FriendshipSvc::new(mock_friendship_repo, Arc::new(mock_user_svc));
            let result = friendship_svc
                .add_friend(my_id, &does_not_know_me.username)
                .await;

            assert!(matches!(result, Ok(false)));
        }
    }

    #[tokio::test]
    async fn gets_all_friends_usernames() {
        let [friend1, me, friend2, friend3] = make_user::all4();

        let friend_usernames = vec![
            friend1.username.clone(),
            friend2.username.clone(),
            friend3.username.clone(),
        ];

        let mut mock_friendship_repo = MockFriendshipStore::new();
        mock_friendship_repo
            .expect_get_friends()
            .with(eq(me.id))
            .once()
            .return_once(move |_| Ok(vec![friend1.id, friend2.id, friend3.id]));

        let mut mock_user_svc = MockUserManager::new();
        mock_user_svc
            .expect_get_by_id()
            .with(eq(friend1.id))
            .once()
            .return_once(|_| Ok(friend1));
        mock_user_svc
            .expect_get_by_id()
            .with(eq(friend2.id))
            .once()
            .return_once(|_| Ok(friend2));
        mock_user_svc
            .expect_get_by_id()
            .with(eq(friend3.id))
            .once()
            .return_once(|_| Ok(friend3));

        let friendship_svc = FriendshipSvc::new(mock_friendship_repo, Arc::new(mock_user_svc));
        let result = friendship_svc
            .get_friends(me.id)
            .await
            .expect("failed to get friends");

        assert_eq!(result, friend_usernames);
    }

    #[tokio::test]
    async fn gets_pending_request_usernames() {
        let [requester3, requester2, requester1, me] = make_user::all4();

        let requester_usernames = vec![
            requester1.username.clone(),
            requester2.username.clone(),
            requester3.username.clone(),
        ];

        let mut mock_friendship_repo = MockFriendshipStore::new();
        mock_friendship_repo
            .expect_get_requests()
            .with(eq(me.id))
            .once()
            .return_once(move |_| Ok(vec![requester1.id, requester2.id, requester3.id]));

        let mut mock_user_svc = MockUserManager::new();
        mock_user_svc
            .expect_get_by_id()
            .with(eq(requester1.id))
            .once()
            .return_once(|_| Ok(requester1));
        mock_user_svc
            .expect_get_by_id()
            .with(eq(requester2.id))
            .once()
            .return_once(|_| Ok(requester2));
        mock_user_svc
            .expect_get_by_id()
            .with(eq(requester3.id))
            .once()
            .return_once(|_| Ok(requester3));

        let friendship_svc = FriendshipSvc::new(mock_friendship_repo, Arc::new(mock_user_svc));
        let result = friendship_svc
            .get_requests(me.id)
            .await
            .expect("failed to get requests");

        assert_eq!(result, requester_usernames);
    }

    #[tokio::test]
    async fn correctly_reports_friendship_as_bool() {
        let ids1 = UserIdPair::new(15, 2).unwrap();
        let ids2 = UserIdPair::new(999, 55).unwrap();
        let ids3 = UserIdPair::new(739, 24252).unwrap();
        let ids4 = UserIdPair::new(9, 10).unwrap();

        let ids2_lesser = ids2.lesser();
        let ids3_greater = ids3.greater();

        let mut mock_friendship_repo = MockFriendshipStore::new();
        mock_friendship_repo
            .expect_get_status()
            .with(eq(ids1.clone()))
            .once()
            .return_once(|_| Ok(FriendshipStatus::Friends));
        mock_friendship_repo
            .expect_get_status()
            .with(eq(ids2.clone()))
            .once()
            .return_once(move |_| Ok(FriendshipStatus::PendingFrom(ids2_lesser)));
        mock_friendship_repo
            .expect_get_status()
            .with(eq(ids3.clone()))
            .once()
            .return_once(move |_| Ok(FriendshipStatus::PendingFrom(ids3_greater)));
        mock_friendship_repo
            .expect_get_status()
            .with(eq(ids4.clone()))
            .once()
            .return_once(|_| Ok(FriendshipStatus::Nil));

        let friendship_svc =
            FriendshipSvc::new(mock_friendship_repo, Arc::new(MockUserManager::new()));

        assert!(friendship_svc.are_friends(&ids1).await.unwrap());
        assert!(!friendship_svc.are_friends(&ids2).await.unwrap());
        assert!(!friendship_svc.are_friends(&ids3).await.unwrap());
        assert!(!friendship_svc.are_friends(&ids4).await.unwrap());
    }
}
