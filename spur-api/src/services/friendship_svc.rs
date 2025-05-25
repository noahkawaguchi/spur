use super::domain_error::{DomainError, FriendshipError};
use crate::{
    handlers::friendship_handlers::FriendshipManager,
    repositories::{friendship_repo::FriendshipStatus, user_repo::UserStore},
    technical_error::TechnicalError,
};
use std::sync::Arc;

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait FriendshipStore: Send + Sync {
    /// Creates a new friend request between the two users.
    ///
    /// - `first_id` should always be less than `second_id`.
    /// - `requester_id`, equal to either `first_id` or `second_id`, indicates who initiated the
    /// request.
    async fn new_request(
        &self,
        first_id: i32,
        second_id: i32,
        requester_id: i32,
    ) -> Result<(), TechnicalError>;

    /// Accepts a pending friend request that involves the two users, regardless of who initiated
    /// it.
    ///
    /// `first_id` should always be less than `second_id`.
    async fn accept_request(&self, first_id: i32, second_id: i32) -> Result<(), TechnicalError>;

    /// Determines the status of the relationship between the two users.
    ///
    /// `first_id` should always be less than `second_id`.
    ///
    /// See [`FriendshipStatus`] for more information on status meanings.
    async fn get_status(
        &self,
        first_id: i32,
        second_id: i32,
    ) -> Result<FriendshipStatus, TechnicalError>;

    /// Retrieves the IDs of all confirmed friends of the user with the provided ID.
    async fn get_friends(&self, id: i32) -> Result<Vec<i32>, TechnicalError>;

    /// Retrieves the IDs of all users who have pending requests to the user with the provided ID.
    async fn get_requests(&self, id: i32) -> Result<Vec<i32>, TechnicalError>;
}

pub struct FriendshipSvc {
    friendship_store: Arc<dyn FriendshipStore>,
    user_store: Arc<dyn UserStore>,
}

impl FriendshipSvc {
    pub fn new(friendship_store: Arc<dyn FriendshipStore>, user_store: Arc<dyn UserStore>) -> Self {
        Self { friendship_store, user_store }
    }
}

#[async_trait::async_trait]
impl FriendshipManager for FriendshipSvc {
    async fn add_friend(
        &self,
        sender_id: i32,
        recipient_username: &str,
    ) -> Result<bool, DomainError> {
        // First find the recipient's ID
        let recipient_id = self
            .user_store
            .get_by_username(recipient_username)
            .await?
            .ok_or(FriendshipError::NonexistentUser)?
            .id;

        // Determine how this pair would be stored in the database
        let (first_id, second_id) = if sender_id < recipient_id {
            (sender_id, recipient_id)
        } else {
            (recipient_id, sender_id)
        };

        // Determine the pair's current status
        match self
            .friendship_store
            .get_status(first_id, second_id)
            .await?
        {
            // Already friends, cannot request to become friends
            FriendshipStatus::Friends => Err(FriendshipError::AlreadyFriends.into()),

            // A request from this sender to this recipient already exists, cannot request again
            FriendshipStatus::PendingFrom(id) if id == sender_id => {
                Err(FriendshipError::AlreadyRequested.into())
            }

            // Already a pending request in the opposite direction, so accept it
            FriendshipStatus::PendingFrom(_) => {
                self.friendship_store
                    .accept_request(first_id, second_id)
                    .await?;

                Ok(true)
            }

            // No existing relationship, create a new request
            FriendshipStatus::Nil => {
                self.friendship_store
                    .new_request(first_id, second_id, sender_id)
                    .await?;

                Ok(false)
            }
        }
    }

    async fn get_friends(&self, id: i32) -> Result<Vec<String>, DomainError> {
        futures::future::try_join_all(
            self.friendship_store
                .get_friends(id)
                .await?
                .into_iter()
                .map(|id| async move { Ok(self.user_store.get_by_id(id).await?.username) }),
        )
        .await
    }

    async fn get_requests(&self, id: i32) -> Result<Vec<String>, DomainError> {
        futures::future::try_join_all(
            self.friendship_store
                .get_requests(id)
                .await?
                .into_iter()
                .map(|id| async move { Ok(self.user_store.get_by_id(id).await?.username) }),
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{models::user::User, repositories::user_repo::MockUserStore};
    use chrono::{Days, Months, Utc};
    use mockall::predicate::eq;

    fn make_user1() -> User {
        User {
            id: 41,
            name: String::from("Friendly Good"),
            email: String::from("good@friend.co"),
            username: String::from("my_friend_5"),
            password_hash: String::from("ab5iub$@1i&g"),
            created_at: Utc::now(),
        }
    }

    fn make_user2() -> User {
        User {
            id: 42,
            name: String::from("Gillian Jill"),
            email: String::from("gillian@jill.org"),
            username: String::from("jill_plus_ian"),
            password_hash: String::from("aab52i4n&$"),
            created_at: Utc::now()
                .checked_sub_days(Days::new(1))
                .expect("failed to subtract one day from now"),
        }
    }

    fn make_user3() -> User {
        User {
            id: 43,
            name: String::from("Harold Old"),
            email: String::from("harold@old.jp"),
            username: String::from("old_hare"),
            password_hash: String::from("ljb42b50%&$@"),
            created_at: Utc::now()
                .checked_sub_months(Months::new(1))
                .expect("failed to subtract one month from now"),
        }
    }

    fn make_user4() -> User {
        User {
            id: 44,
            name: String::from("Greg Egg"),
            email: String::from("egg_greg@egg.gg"),
            username: String::from("greg_the_egg"),
            password_hash: String::from("5%2b@$$@bu"),
            created_at: Utc::now()
                .checked_sub_months(Months::new(6))
                .expect("failed to subtract six months from now"),
        }
    }

    mod add_friend {
        use super::*;

        #[tokio::test]
        async fn errors_for_nonexistent_user() {
            let recipient_username = "not_real";

            let mut mock_user_repo = MockUserStore::new();
            mock_user_repo
                .expect_get_by_username()
                .with(eq(recipient_username))
                .once()
                .return_once(|_| Ok(None));

            let mock_friendship_repo = MockFriendshipStore::new();

            let friendship_svc =
                FriendshipSvc::new(Arc::new(mock_friendship_repo), Arc::new(mock_user_repo));
            let result = friendship_svc.add_friend(42, recipient_username).await;

            assert!(matches!(
                result,
                Err(DomainError::Friendship(FriendshipError::NonexistentUser)),
            ));
        }

        #[tokio::test]
        async fn disallows_sending_a_friend_request_to_a_friend() {
            let my_friend = make_user1();
            let my_friend_clone = my_friend.clone();
            let my_id = my_friend.id - 1;

            let mut mock_user_repo = MockUserStore::new();
            mock_user_repo
                .expect_get_by_username()
                .with(eq(my_friend.username.clone()))
                .once()
                .return_once(|_| Ok(Some(my_friend_clone)));

            let mut mock_friendship_repo = MockFriendshipStore::new();
            mock_friendship_repo
                .expect_get_status()
                .with(eq(my_id), eq(my_friend.id))
                .once()
                .return_once(|_, _| Ok(FriendshipStatus::Friends));

            let friendship_svc =
                FriendshipSvc::new(Arc::new(mock_friendship_repo), Arc::new(mock_user_repo));
            let result = friendship_svc.add_friend(my_id, &my_friend.username).await;

            assert!(matches!(
                result,
                Err(DomainError::Friendship(FriendshipError::AlreadyFriends)),
            ));
        }

        #[tokio::test]
        async fn disallows_duplicate_friend_requests() {
            let desired_friend = make_user2();
            let desired_friend_clone = desired_friend.clone();
            let my_id = desired_friend.id + 3;

            let mut mock_user_repo = MockUserStore::new();
            mock_user_repo
                .expect_get_by_username()
                .with(eq(desired_friend.username.clone()))
                .once()
                .return_once(|_| Ok(Some(desired_friend_clone)));

            let mut mock_friendship_repo = MockFriendshipStore::new();
            mock_friendship_repo
                .expect_get_status()
                .with(eq(desired_friend.id), eq(my_id))
                .once()
                .return_once(move |_, _| Ok(FriendshipStatus::PendingFrom(my_id)));

            let friendship_svc =
                FriendshipSvc::new(Arc::new(mock_friendship_repo), Arc::new(mock_user_repo));
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
            let added_me = make_user3();
            let added_me_clone = added_me.clone();
            let my_id = added_me.id + 100;

            let mut mock_user_repo = MockUserStore::new();
            mock_user_repo
                .expect_get_by_username()
                .with(eq(added_me.username.clone()))
                .once()
                .return_once(|_| Ok(Some(added_me_clone)));

            let mut mock_friendship_repo = MockFriendshipStore::new();
            mock_friendship_repo
                .expect_get_status()
                .with(eq(added_me.id), eq(my_id))
                .once()
                .return_once(move |_, _| Ok(FriendshipStatus::PendingFrom(added_me.id)));
            mock_friendship_repo
                .expect_accept_request()
                .with(eq(added_me.id), eq(my_id))
                .once()
                .return_once(|_, _| Ok(()));

            let friendship_svc =
                FriendshipSvc::new(Arc::new(mock_friendship_repo), Arc::new(mock_user_repo));
            let result = friendship_svc.add_friend(my_id, &added_me.username).await;

            assert!(matches!(result, Ok(true)));
        }

        #[tokio::test]
        async fn creates_a_request_if_no_relationship() {
            let does_not_know_me = make_user4();
            let does_not_know_me_clone = does_not_know_me.clone();
            let my_id = does_not_know_me.id - 7;

            let mut mock_user_repo = MockUserStore::new();
            mock_user_repo
                .expect_get_by_username()
                .with(eq(does_not_know_me.username.clone()))
                .once()
                .return_once(|_| Ok(Some(does_not_know_me_clone)));

            let mut mock_friendship_repo = MockFriendshipStore::new();
            mock_friendship_repo
                .expect_get_status()
                .with(eq(my_id), eq(does_not_know_me.id))
                .once()
                .return_once(move |_, _| Ok(FriendshipStatus::Nil));
            mock_friendship_repo.expect_accept_request().never();
            mock_friendship_repo
                .expect_new_request()
                .with(eq(my_id), eq(does_not_know_me.id), eq(my_id))
                .once()
                .return_once(|_, _, _| Ok(()));

            let friendship_svc =
                FriendshipSvc::new(Arc::new(mock_friendship_repo), Arc::new(mock_user_repo));
            let result = friendship_svc
                .add_friend(my_id, &does_not_know_me.username)
                .await;

            assert!(matches!(result, Ok(false)));
        }
    }

    #[tokio::test]
    async fn gets_all_friends_usernames() {
        let (friend1, me, friend2, friend3) =
            (make_user1(), make_user2(), make_user3(), make_user4());

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

        let mut mock_user_repo = MockUserStore::new();
        mock_user_repo
            .expect_get_by_id()
            .with(eq(friend1.id))
            .once()
            .return_once(|_| Ok(friend1));
        mock_user_repo
            .expect_get_by_id()
            .with(eq(friend2.id))
            .once()
            .return_once(|_| Ok(friend2));
        mock_user_repo
            .expect_get_by_id()
            .with(eq(friend3.id))
            .once()
            .return_once(|_| Ok(friend3));

        let friendship_svc =
            FriendshipSvc::new(Arc::new(mock_friendship_repo), Arc::new(mock_user_repo));
        let result = friendship_svc
            .get_friends(me.id)
            .await
            .expect("failed to get friends");

        assert_eq!(result, friend_usernames);
    }

    #[tokio::test]
    async fn gets_pending_request_usernames() {
        let (requester3, requester2, requester1, me) =
            (make_user1(), make_user2(), make_user3(), make_user4());

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

        let mut mock_user_repo = MockUserStore::new();
        mock_user_repo
            .expect_get_by_id()
            .with(eq(requester1.id))
            .once()
            .return_once(|_| Ok(requester1));
        mock_user_repo
            .expect_get_by_id()
            .with(eq(requester2.id))
            .once()
            .return_once(|_| Ok(requester2));
        mock_user_repo
            .expect_get_by_id()
            .with(eq(requester3.id))
            .once()
            .return_once(|_| Ok(requester3));

        let friendship_svc =
            FriendshipSvc::new(Arc::new(mock_friendship_repo), Arc::new(mock_user_repo));
        let result = friendship_svc
            .get_requests(me.id)
            .await
            .expect("failed to get requests");

        assert_eq!(result, requester_usernames);
    }
}
