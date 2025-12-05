use super::api_result;
use crate::{
    api::{
        dto::{
            requests::AddFriendRequest,
            responses::{PostResponse, SuccessResponse},
        },
        validated_json::ValidatedJson,
    },
    app_services::MutateFriendshipByUsername,
    map_into::MapInto,
    read_models::SocialRead,
    state::AppState,
};
use axum::{
    Extension, Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use std::sync::Arc;

#[allow(clippy::needless_for_each, clippy::wildcard_imports)]
pub mod docs {
    use super::*;

    #[derive(utoipa::OpenApi)]
    #[openapi(paths(add_friend, list_friends, list_requests, friend_posts))]
    pub struct FriendsDoc;
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(add_friend).get(list_friends))
        .route("/requests", get(list_requests))
        .route("/posts", get(friend_posts))
}

/// Creates a new friend request or accepts an existing friend request.
#[utoipa::path(
    post,
    tag = "friends",
    path = "",
    security(("jwt" = [])),
    request_body = AddFriendRequest,
    responses(
        (
            status = StatusCode::OK,
            body = SuccessResponse,
            description = "You are now friends",
        ),
        (
            status = StatusCode::CREATED,
            body = SuccessResponse,
            description = "A new friend request was created",
        ),
    ),
)]
async fn add_friend(
    mutate_friendship_by_username: State<Arc<dyn MutateFriendshipByUsername>>,
    Extension(requester_id): Extension<i32>,
    payload: ValidatedJson<AddFriendRequest>,
) -> api_result!(SuccessResponse) {
    // Try to add the friend
    let became_friends = mutate_friendship_by_username
        .add_friend_by_username(requester_id, &payload.recipient_username)
        .await?;

    let (status_code, message) = if became_friends {
        (
            StatusCode::OK,
            format!("You are now friends with {}", payload.recipient_username),
        )
    } else {
        (
            StatusCode::CREATED,
            format!("Created a friend request to {}", payload.recipient_username),
        )
    };

    Ok((status_code, Json(SuccessResponse { message })))
}

/// Retrieves the usernames of the requester's friends.
#[utoipa::path(
    get,
    tag = "friends",
    path = "",
    security(("jwt" = [])),
    responses(
        (
            status = StatusCode::OK,
            body = Vec<String>,
            example = json!(["alice", "bob", "cool_user_123", "DinnerWithDiana"]),
        ),
    ),
)]
async fn list_friends(
    social_read: State<Arc<dyn SocialRead>>,
    Extension(requester_id): Extension<i32>,
) -> api_result!(Vec<String>) {
    Ok((
        StatusCode::OK,
        Json(social_read.friend_usernames(requester_id).await?),
    ))
}

/// Retrieves the usernames of users who have pending friend requests to the requester.
#[utoipa::path(
    get,
    tag = "friends",
    path = "/requests",
    security(("jwt" = [])),
    responses(
        (
            status = StatusCode::OK,
            body = Vec<String>,
            example = json!(["Ako", "bruno", "c4554ndr4", "dan-o", "__EVELYN__"]),
        ),
    ),
)]
async fn list_requests(
    social_read: State<Arc<dyn SocialRead>>,
    Extension(requester_id): Extension<i32>,
) -> api_result!(Vec<String>) {
    Ok((
        StatusCode::OK,
        Json(social_read.pending_requests(requester_id).await?),
    ))
}

/// Retrieves all posts written by the requester's friends.
#[utoipa::path(
    get,
    tag = "friends",
    path = "/posts",
    security(("jwt" = [])),
    responses((status = StatusCode::OK, body = Vec<PostResponse>)),
)]
async fn friend_posts(
    social_read: State<Arc<dyn SocialRead>>,
    Extension(requester_id): Extension<i32>,
) -> api_result!(Vec<PostResponse>) {
    Ok((
        StatusCode::OK,
        Json(social_read.friend_posts(requester_id).await?.map_into()),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        api::dto::responses::ErrorResponse,
        app_services::MockMutateFriendshipByUsername,
        domain::friendship::error::FriendshipError,
        read_models::{MockSocialRead, ReadError},
        test_utils::{
            dummy_data::post_with_author,
            http_bodies::{deserialize_body, serialize_body},
            tokio_test,
        },
    };
    use anyhow::anyhow;
    use axum::{
        body::Body,
        http::{Method, Request, header::CONTENT_TYPE},
    };
    use mockall::predicate::eq;
    use tower::ServiceExt;

    mod add_friend {
        use super::*;

        #[test]
        fn reports_successfully_becoming_friends() {
            tokio_test(async {
                let requester_id = 42;
                let recipient_username = "jonathan_johnson";

                let mut mock_svc = MockMutateFriendshipByUsername::new();
                mock_svc
                    .expect_add_friend_by_username()
                    .with(eq(requester_id), eq(recipient_username))
                    .once()
                    .return_once(|_, _| Ok(true));

                let state = AppState {
                    mutate_friendship_by_username: Arc::new(mock_svc),
                    ..Default::default()
                };
                let app = super::routes().with_state(state);

                let req_body = serialize_body(&AddFriendRequest {
                    recipient_username: String::from(recipient_username),
                });

                let mut req = Request::builder()
                    .method(Method::POST)
                    .uri("/")
                    .header(CONTENT_TYPE, "application/json")
                    .body(req_body)
                    .unwrap();

                req.extensions_mut().insert(requester_id);

                let resp = app.oneshot(req).await.unwrap();
                assert_eq!(resp.status(), StatusCode::OK);

                let resp_body = deserialize_body::<SuccessResponse>(resp).await;
                let expected = SuccessResponse {
                    message: format!("You are now friends with {recipient_username}"),
                };
                assert_eq!(expected, resp_body);
            });
        }

        #[test]
        fn reports_successfully_creating_a_friend_request() {
            tokio_test(async {
                let requester_id = 43;
                let recipient_username = "jane_sane";

                let mut mock_svc = MockMutateFriendshipByUsername::new();
                mock_svc
                    .expect_add_friend_by_username()
                    .with(eq(requester_id), eq(recipient_username))
                    .once()
                    .return_once(|_, _| Ok(false));

                let state = AppState {
                    mutate_friendship_by_username: Arc::new(mock_svc),
                    ..Default::default()
                };
                let app = super::routes().with_state(state);

                let req_body = serialize_body(&AddFriendRequest {
                    recipient_username: String::from(recipient_username),
                });

                let mut req = Request::builder()
                    .method(Method::POST)
                    .uri("/")
                    .header(CONTENT_TYPE, "application/json")
                    .body(req_body)
                    .unwrap();

                req.extensions_mut().insert(requester_id);

                let resp = app.oneshot(req).await.unwrap();
                assert_eq!(resp.status(), StatusCode::CREATED);

                let resp_body = deserialize_body::<SuccessResponse>(resp).await;
                let expected = SuccessResponse {
                    message: format!("Created a friend request to {recipient_username}"),
                };
                assert_eq!(expected, resp_body);
            });
        }

        #[test]
        fn translates_errors() {
            tokio_test(async {
                let requester_id = 44;
                let recipient_username = "malcolm_holmes";

                let mut mock_svc = MockMutateFriendshipByUsername::new();
                mock_svc
                    .expect_add_friend_by_username()
                    .with(eq(requester_id), eq(recipient_username))
                    .once()
                    .return_once(|_, _| Err(FriendshipError::AlreadyRequested));

                let state = AppState {
                    mutate_friendship_by_username: Arc::new(mock_svc),
                    ..Default::default()
                };
                let app = super::routes().with_state(state);

                let req_body = serialize_body(&AddFriendRequest {
                    recipient_username: String::from(recipient_username),
                });

                let mut req = Request::builder()
                    .method(Method::POST)
                    .uri("/")
                    .header(CONTENT_TYPE, "application/json")
                    .body(req_body)
                    .unwrap();

                req.extensions_mut().insert(requester_id);

                let resp = app.oneshot(req).await.unwrap();
                assert_eq!(resp.status(), StatusCode::CONFLICT);

                let resp_body = deserialize_body::<ErrorResponse>(resp).await;
                let expected = ErrorResponse {
                    error: String::from("Pending friend request to this user already exists"),
                };
                assert_eq!(expected, resp_body);
            });
        }
    }

    mod list_friends {
        use super::*;

        #[test]
        fn lists_retrieved_usernames() {
            tokio_test(async {
                let requester_id = 44;
                let friends = vec![
                    String::from("Alice"),
                    String::from("Bob"),
                    String::from("Callahan"),
                ];
                let friends_clone = friends.clone();

                let mut mock_social_read = MockSocialRead::new();
                mock_social_read
                    .expect_friend_usernames()
                    .with(eq(requester_id))
                    .once()
                    .return_once(|_| Ok(friends_clone));

                let state =
                    AppState { social_read: Arc::new(mock_social_read), ..Default::default() };
                let app = super::routes().with_state(state);

                let mut req = Request::builder()
                    .method(Method::GET)
                    .uri("/")
                    .body(Body::empty())
                    .unwrap();

                req.extensions_mut().insert(requester_id);

                let resp = app.oneshot(req).await.unwrap();
                assert_eq!(resp.status(), StatusCode::OK);

                let resp_body = deserialize_body::<Vec<String>>(resp).await;
                assert_eq!(friends, resp_body);
            });
        }

        #[test]
        fn translates_errors() {
            tokio_test(async {
                let requester_id = 450;

                let mut mock_social_read = MockSocialRead::new();
                mock_social_read
                    .expect_friend_usernames()
                    .with(eq(requester_id))
                    .once()
                    .return_once(|_| Err(ReadError::NotFound));

                let state =
                    AppState { social_read: Arc::new(mock_social_read), ..Default::default() };
                let app = super::routes().with_state(state);

                let mut req = Request::builder()
                    .method(Method::GET)
                    .uri("/")
                    .body(Body::empty())
                    .unwrap();

                req.extensions_mut().insert(requester_id);

                let resp = app.oneshot(req).await.unwrap();
                assert_eq!(resp.status(), StatusCode::NOT_FOUND);

                let resp_body = deserialize_body::<ErrorResponse>(resp).await;
                let expected = ErrorResponse { error: String::from("Not found") };
                assert_eq!(expected, resp_body);
            });
        }
    }

    mod list_requests {
        use super::*;

        #[test]
        fn lists_retrieved_usernames() {
            tokio_test(async {
                let requester_id = 5;
                let requesters = vec![
                    String::from("Dirk"),
                    String::from("Elaine"),
                    String::from("Francesca"),
                ];
                let requesters_clone = requesters.clone();

                let mut mock_social_read = MockSocialRead::new();
                mock_social_read
                    .expect_pending_requests()
                    .with(eq(requester_id))
                    .once()
                    .return_once(|_| Ok(requesters_clone));

                let state =
                    AppState { social_read: Arc::new(mock_social_read), ..Default::default() };
                let app = super::routes().with_state(state);

                let mut req = Request::builder()
                    .method(Method::GET)
                    .uri("/requests")
                    .body(Body::empty())
                    .unwrap();

                req.extensions_mut().insert(requester_id);

                let resp = app.oneshot(req).await.unwrap();
                assert_eq!(resp.status(), StatusCode::OK);

                let resp_body = deserialize_body::<Vec<String>>(resp).await;
                assert_eq!(requesters, resp_body);
            });
        }

        #[test]
        fn translates_errors() {
            tokio_test(async {
                let requester_id = 56;

                let mut mock_social_read = MockSocialRead::new();
                mock_social_read
                    .expect_pending_requests()
                    .with(eq(requester_id))
                    .once()
                    .return_once(|_| Err(ReadError::Technical(anyhow!("something went wrong!"))));

                let state =
                    AppState { social_read: Arc::new(mock_social_read), ..Default::default() };
                let app = super::routes().with_state(state);

                let mut req = Request::builder()
                    .method(Method::GET)
                    .uri("/requests")
                    .body(Body::empty())
                    .unwrap();

                req.extensions_mut().insert(requester_id);

                let resp = app.oneshot(req).await.unwrap();
                assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);

                let resp_body = deserialize_body::<ErrorResponse>(resp).await;
                let expected = ErrorResponse { error: String::from("internal server error") };
                assert_eq!(expected, resp_body);
            });
        }
    }

    mod friend_posts {
        use super::*;

        #[test]
        fn lists_friend_posts() {
            tokio_test(async {
                let requester_id = 557;
                let posts = post_with_author::all3();
                let posts_clone = posts.clone();

                let mut mock_social_read = MockSocialRead::new();
                mock_social_read
                    .expect_friend_posts()
                    .with(eq(requester_id))
                    .once()
                    .return_once(|_| Ok(Vec::from(posts_clone)));

                let state =
                    AppState { social_read: Arc::new(mock_social_read), ..Default::default() };
                let app = super::routes().with_state(state);

                let mut req = Request::builder()
                    .method(Method::GET)
                    .uri("/posts")
                    .body(Body::empty())
                    .unwrap();

                req.extensions_mut().insert(requester_id);

                let resp = app.oneshot(req).await.unwrap();
                assert_eq!(resp.status(), StatusCode::OK);

                let resp_body = deserialize_body::<Vec<PostResponse>>(resp).await;
                assert_eq!(posts.map_into::<Vec<PostResponse>>(), resp_body);
            });
        }

        #[test]
        fn translates_errors() {
            tokio_test(async {
                let requester_id = 915;

                let mut mock_social_read = MockSocialRead::new();
                mock_social_read
                    .expect_friend_posts()
                    .with(eq(requester_id))
                    .once()
                    .return_once(|_| Err(ReadError::NotFound));

                let state =
                    AppState { social_read: Arc::new(mock_social_read), ..Default::default() };
                let app = super::routes().with_state(state);

                let mut req = Request::builder()
                    .method(Method::GET)
                    .uri("/posts")
                    .body(Body::empty())
                    .unwrap();

                req.extensions_mut().insert(requester_id);

                let resp = app.oneshot(req).await.unwrap();
                assert_eq!(resp.status(), StatusCode::NOT_FOUND);

                let resp_body = deserialize_body::<ErrorResponse>(resp).await;
                let expected = ErrorResponse { error: String::from("Not found") };
                assert_eq!(expected, resp_body);
            });
        }
    }
}
