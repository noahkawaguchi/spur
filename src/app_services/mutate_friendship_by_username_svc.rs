use crate::{
    app_services::{
        MutateFriendshipByUsername,
        uow::{Tx, UnitOfWork},
    },
    domain::{
        friendship::{
            FriendshipRepo, FriendshipStatus, error::FriendshipError, user_id_pair::UserIdPair,
        },
        user::UserRepo,
    },
};

pub struct MutateFriendshipByUsernameSvc<Uo, Us, F> {
    uow: Uo,
    user_repo: Us,
    friendship_repo: F,
}

impl<Uo, Us, F> MutateFriendshipByUsernameSvc<Uo, Us, F> {
    pub const fn new(uow: Uo, user_repo: Us, friendship_repo: F) -> Self {
        Self { uow, user_repo, friendship_repo }
    }
}

#[async_trait::async_trait]
impl<Uo, Us, F> MutateFriendshipByUsername for MutateFriendshipByUsernameSvc<Uo, Us, F>
where
    Uo: UnitOfWork,
    Us: UserRepo,
    F: FriendshipRepo,
{
    async fn add_friend_by_username(
        &self,
        sender_id: i32,
        recipient_username: &str,
    ) -> Result<bool, FriendshipError> {
        let mut tx = self.uow.begin_uow().await?;

        // Find the recipient's ID and create an ID pair
        let recipient_id = self
            .user_repo
            .get_by_username_exclusive(tx.exec(), recipient_username)
            .await?
            .ok_or(FriendshipError::NonexistentUser)?
            .id;

        let ids = UserIdPair::new(sender_id, recipient_id)?;

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
    use crate::test_utils::{
        dummy_data,
        fake_db::FakeUow,
        mock_repos::{MockFriendshipRepo, MockUserRepo},
        tokio_test,
    };
    use anyhow::Result;

    mod add_friend {
        use super::*;

        #[test]
        fn disallows_sending_a_friend_request_to_a_friend() -> Result<()> {
            tokio_test(async {
                let my_friend = dummy_data::user::number1();
                let my_friend_username_clone = my_friend.username.clone();
                let my_friend_clone = my_friend.clone();
                let my_id = my_friend.id - 1;
                let ids = UserIdPair::new(my_id, my_friend.id)?;

                let mock_user_repo = MockUserRepo {
                    get_by_username_exclusive: Some(Box::new(move |passed_username| {
                        assert_eq!(my_friend_username_clone, passed_username);
                        Ok(Some(my_friend_clone.clone()))
                    })),
                    ..Default::default()
                };

                let mock_friendship_repo = MockFriendshipRepo {
                    get_status: Some(Box::new(move |&passed_ids| {
                        assert_eq!(ids, passed_ids);
                        Ok(FriendshipStatus::Friends)
                    })),
                    ..Default::default()
                };

                let (fake_uow, probe) = FakeUow::with_probe()?;

                let friendship_svc = MutateFriendshipByUsernameSvc::new(
                    fake_uow,
                    mock_user_repo,
                    mock_friendship_repo,
                );
                let result = friendship_svc
                    .add_friend_by_username(my_id, &my_friend.username)
                    .await;

                assert!(matches!(result, Err(FriendshipError::AlreadyFriends)));
                assert!(probe.commit_called());

                Ok(())
            })
        }

        #[test]
        fn disallows_duplicate_friend_requests() -> Result<()> {
            tokio_test(async {
                let desired_friend = dummy_data::user::number2()?;
                let desired_friend_clone = desired_friend.clone();
                let desired_friend_username_clone = desired_friend.username.clone();
                let my_id = desired_friend.id + 3;
                let ids = UserIdPair::new(my_id, desired_friend.id)?;

                let mock_user_repo = MockUserRepo {
                    get_by_username_exclusive: Some(Box::new(move |passed_username| {
                        assert_eq!(desired_friend_username_clone, passed_username);
                        Ok(Some(desired_friend_clone.clone()))
                    })),
                    ..Default::default()
                };

                let mock_friendship_repo = MockFriendshipRepo {
                    get_status: Some(Box::new(move |&passed_ids| {
                        assert_eq!(passed_ids, ids);
                        Ok(FriendshipStatus::PendingFrom(my_id))
                    })),
                    ..Default::default()
                };

                let (fake_uow, probe) = FakeUow::with_probe()?;

                let friendship_svc = MutateFriendshipByUsernameSvc::new(
                    fake_uow,
                    mock_user_repo,
                    mock_friendship_repo,
                );
                let result = friendship_svc
                    .add_friend_by_username(my_id, &desired_friend.username)
                    .await;

                assert!(matches!(result, Err(FriendshipError::AlreadyRequested)));
                assert!(probe.commit_called());

                Ok(())
            })
        }

        #[test]
        fn accepts_a_friend_request_in_the_opposite_direction() -> Result<()> {
            tokio_test(async {
                let added_me = dummy_data::user::number3()?;
                let added_me_clone = added_me.clone();
                let added_me_username_clone = added_me.username.clone();
                let my_id = added_me.id + 100;
                let ids = UserIdPair::new(my_id, added_me.id)?;

                let mock_user_repo = MockUserRepo {
                    get_by_username_exclusive: Some(Box::new(move |passed_username| {
                        assert_eq!(added_me_username_clone, passed_username);
                        Ok(Some(added_me_clone.clone()))
                    })),
                    ..Default::default()
                };

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

                let (fake_uow, probe) = FakeUow::with_probe()?;

                let friendship_svc = MutateFriendshipByUsernameSvc::new(
                    fake_uow,
                    mock_user_repo,
                    mock_friendship_repo,
                );
                let result = friendship_svc
                    .add_friend_by_username(my_id, &added_me.username)
                    .await;

                assert!(matches!(result, Ok(true)));
                assert!(probe.commit_called());

                Ok(())
            })
        }

        #[test]
        fn creates_a_request_if_no_relationship() -> Result<()> {
            tokio_test(async {
                let does_not_know_me = dummy_data::user::number4()?;
                let does_not_know_me_clone = does_not_know_me.clone();
                let does_not_know_me_username_clone = does_not_know_me.username.clone();
                let my_id = does_not_know_me.id - 7;
                let ids = UserIdPair::new(my_id, does_not_know_me.id)?;

                let mock_user_svc = MockUserRepo {
                    get_by_username_exclusive: Some(Box::new(move |passed_username| {
                        assert_eq!(does_not_know_me_username_clone, passed_username);
                        Ok(Some(does_not_know_me_clone.clone()))
                    })),
                    ..Default::default()
                };

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

                let (fake_uow, probe) = FakeUow::with_probe()?;

                let friendship_svc = MutateFriendshipByUsernameSvc::new(
                    fake_uow,
                    mock_user_svc,
                    mock_friendship_repo,
                );
                let result = friendship_svc
                    .add_friend_by_username(my_id, &does_not_know_me.username)
                    .await;

                assert!(matches!(result, Ok(false)));
                assert!(probe.commit_called());

                Ok(())
            })
        }
    }
}
