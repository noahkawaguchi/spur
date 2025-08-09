// use crate::{
//     domain::{
//         content::{
//             error::ContentError,
//             service::{ContentManager, PostManager},
//         },
//         error::DomainError,
//         friendship::{service::FriendshipManager, user_id_pair::UserIdPair},
//         user::UserManager,
//     },
//     models::prompt::PromptWithAuthor,
// };
// use std::sync::Arc;

// pub struct ContentSvc {
//     users: Arc<dyn UserManager>,
//     friendships: Arc<dyn FriendshipManager>,
//     prompts: Arc<dyn PromptManager>,
//     posts: Arc<dyn PostManager>,
// }

// impl ContentSvc {
//     pub const fn new(
//         users: Arc<dyn UserManager>,
//         friendships: Arc<dyn FriendshipManager>,
//         prompts: Arc<dyn PromptManager>,
//         posts: Arc<dyn PostManager>,
//     ) -> Self {
//         Self { users, friendships, prompts, posts }
//     }
// }

// #[async_trait::async_trait]
// impl ContentManager for ContentSvc {
//     async fn own_content(
//         &self,
//         user_id: i32,
//     ) -> Result<(Vec<PromptWithAuthor>, Vec<PostWithPrompt>), DomainError> {
//         let prompts = self.prompts.single_user_prompts(user_id).await?;
//         let posts = self.posts.single_user_posts(user_id).await?;
//         Ok((prompts, posts))
//     }

//     async fn specific_friend_content(
//         &self,
//         requester_id: i32,
//         friend_username: &str,
//     ) -> Result<(Vec<PromptWithAuthor>, Vec<PostWithPrompt>), DomainError> {
//         let friend_id = self.users.get_by_username(friend_username).await?.id;

//         // Must be friends to see someone's content
//         if self
//             .friendships
//             .are_friends(&UserIdPair::new(requester_id, friend_id)?)
//             .await?
//         {
//             let prompts = self.prompts.single_user_prompts(friend_id).await?;
//             let posts = self.posts.single_user_posts(friend_id).await?;
//             Ok((prompts, posts))
//         } else {
//             Err(ContentError::NotFriends.into())
//         }
//     }

//     async fn all_friend_content(
//         &self,
//         user_id: i32,
//     ) -> Result<(Vec<PromptWithAuthor>, Vec<PostWithPrompt>), DomainError> {
//         let prompts = self.prompts.all_friend_prompts(user_id).await?;
//         let posts = self.posts.all_friend_posts(user_id).await?;
//         Ok((prompts, posts))
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::{
//         domain::{
//             content::service::{MockPostManager, MockPromptManager},
//             friendship::service::MockFriendshipManager,
//             user::MockUserManager,
//         },
//         test_utils::dummy_data,
//     };
//     use mockall::predicate::eq;

//     #[tokio::test]
//     async fn disallows_seeing_content_if_not_friends() {
//         let non_friend = dummy_data::user::number1();
//         let non_friend_clone = non_friend.clone();
//         let requester_id = non_friend.id + 55;

//         let mut mock_user_svc = MockUserManager::new();
//         mock_user_svc
//             .expect_get_by_username()
//             .with(eq(non_friend.username.clone()))
//             .once()
//             .return_once(|_| Ok(non_friend_clone));

//         let mut mock_friendship_svc = MockFriendshipManager::new();
//         mock_friendship_svc
//             .expect_are_friends()
//             .with(eq(UserIdPair::new(non_friend.id, requester_id).unwrap()))
//             .once()
//             .return_once(|_| Ok(false));

//         let content_svc = ContentSvc::new(
//             Arc::new(mock_user_svc),
//             Arc::new(mock_friendship_svc),
//             Arc::new(MockPromptManager::new()),
//             Arc::new(MockPostManager::new()),
//         );

//         let result = content_svc
//             .specific_friend_content(requester_id, &non_friend.username)
//             .await;

//         assert!(matches!(
//             result,
//             Err(DomainError::Content(ContentError::NotFriends))
//         ));
//     }

//     #[tokio::test]
//     async fn gets_content_if_friends() {
//         let friend = dummy_data::user::number2();
//         let friend_clone = friend.clone();
//         let requester_id = friend.id + 25;

//         let friend_prompts = dummy_data::prompt_with_author::all().to_vec();
//         let friend_prompts_clone = friend_prompts.clone();
//         let friend_posts = dummy_data::post_with_prompt::all().to_vec();
//         let friend_posts_clone = friend_posts.clone();

//         let mut mock_user_svc = MockUserManager::new();
//         mock_user_svc
//             .expect_get_by_username()
//             .with(eq(friend.username.clone()))
//             .once()
//             .return_once(|_| Ok(friend_clone));

//         let mut mock_friendship_svc = MockFriendshipManager::new();
//         mock_friendship_svc
//             .expect_are_friends()
//             .with(eq(UserIdPair::new(friend.id, requester_id).unwrap()))
//             .once()
//             .return_once(|_| Ok(true));

//         let mut mock_prompt_svc = MockPromptManager::new();
//         mock_prompt_svc
//             .expect_single_user_prompts()
//             .with(eq(friend.id))
//             .once()
//             .return_once(|_| Ok(friend_prompts_clone));

//         let mut mock_post_svc = MockPostManager::new();
//         mock_post_svc
//             .expect_single_user_posts()
//             .with(eq(friend.id))
//             .once()
//             .return_once(|_| Ok(friend_posts_clone));

//         let content_svc = ContentSvc::new(
//             Arc::new(mock_user_svc),
//             Arc::new(mock_friendship_svc),
//             Arc::new(mock_prompt_svc),
//             Arc::new(mock_post_svc),
//         );

//         let result = content_svc
//             .specific_friend_content(requester_id, &friend.username)
//             .await
//             .expect("failed to get friend's content");

//         assert_eq!(result, (friend_prompts, friend_posts));
//     }

//     // Determined that `own_content` and `all_friend_content` do not need to be tested at this
//     // point because they just combine the results of the prompt and post functions.
// }
