use super::api_result;
use crate::{
    api::{
        dto::{requests::CreatePostRequest, responses::PostResponse},
        validated_json::ValidatedJson,
    },
    domain::post::PostSvc,
    map_into::MapInto,
    read_models::PostWithAuthorRead,
    state::AppState,
};
use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use std::sync::Arc;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_new))
        .route("/{post_id}", get(by_post_id))
        .route("/{post_id}/children", get(child_posts))
        .route("/user/{author_username}", get(specific_user_posts))
        .route("/me", get(own_posts))
}

/// Creates a new post.
async fn create_new(
    post_svc: State<Arc<dyn PostSvc>>,
    Extension(requester_id): Extension<i32>,
    payload: ValidatedJson<CreatePostRequest>,
) -> api_result!() {
    post_svc
        .create_new(requester_id, payload.parent_id, &payload.body)
        .await?;
    Ok(StatusCode::CREATED)
}

/// Retrieves a post using its ID.
async fn by_post_id(
    post_with_author_read: State<Arc<dyn PostWithAuthorRead>>,
    Path(post_id): Path<i32>,
) -> api_result!(PostResponse) {
    Ok((
        StatusCode::OK,
        Json(post_with_author_read.by_post_id(post_id).await?.into()),
    ))
}

/// Retrieves the children of the post with the provided ID.
async fn child_posts(
    post_with_author_read: State<Arc<dyn PostWithAuthorRead>>,
    Path(parent_id): Path<i32>,
) -> api_result!(Vec<PostResponse>) {
    Ok((
        StatusCode::OK,
        Json(post_with_author_read.by_parent(parent_id).await?.map_into()),
    ))
}

/// Retrieves posts written by the user with the specified username.
async fn specific_user_posts(
    post_with_author_read: State<Arc<dyn PostWithAuthorRead>>,
    Path(author_username): Path<String>,
) -> api_result!(Vec<PostResponse>) {
    Ok((
        StatusCode::OK,
        Json(
            post_with_author_read
                .by_author_username(&author_username)
                .await?
                .map_into(),
        ),
    ))
}

/// Retrieves the requester's own posts.
async fn own_posts(
    post_with_author_read: State<Arc<dyn PostWithAuthorRead>>,
    Extension(requester_id): Extension<i32>,
) -> api_result!(Vec<PostResponse>) {
    Ok((
        StatusCode::OK,
        Json(
            post_with_author_read
                .by_author(requester_id)
                .await?
                .map_into(),
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        api::dto::responses::ErrorResponse,
        domain::post::{MockPostSvc, error::PostError},
        read_models::{MockPostWithAuthorRead, ReadError},
        test_utils::{
            dummy_data::post_with_author,
            http_bodies::{deserialize_body, serialize_body},
        },
    };
    use anyhow::anyhow;
    use axum::{
        body::Body,
        http::{Method, Request, header::CONTENT_TYPE},
    };
    use mockall::predicate::eq;
    use tower::ServiceExt;

    mod create_new {
        use super::*;

        #[tokio::test]
        async fn reports_successfully_creating_a_post() {
            let requester_id = 93;
            let payload = CreatePostRequest {
                parent_id: 925,
                body: String::from("I want to create this post"),
            };

            let mut mock_post_svc = MockPostSvc::new();
            mock_post_svc
                .expect_create_new()
                .with(
                    eq(requester_id),
                    eq(payload.parent_id),
                    eq(payload.body.clone()),
                )
                .once()
                .return_once(|_, _, _| Ok(()));

            let state = AppState { post_svc: Arc::new(mock_post_svc), ..Default::default() };
            let app = super::routes().with_state(state);

            let mut req = Request::builder()
                .method(Method::POST)
                .uri("/")
                .header(CONTENT_TYPE, "application/json")
                .body(serialize_body(&payload))
                .unwrap();

            req.extensions_mut().insert(requester_id);

            let resp = app.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::CREATED);
        }

        #[tokio::test]
        async fn translates_errors() {
            let requester_id = 2052;
            let payload =
                CreatePostRequest { parent_id: 275, body: String::from("This will fail") };

            let mut mock_post_svc = MockPostSvc::new();
            mock_post_svc
                .expect_create_new()
                .with(
                    eq(requester_id),
                    eq(payload.parent_id),
                    eq(payload.body.clone()),
                )
                .once()
                .return_once(|_, _, _| Err(PostError::DeletedParent));

            let state = AppState { post_svc: Arc::new(mock_post_svc), ..Default::default() };
            let app = super::routes().with_state(state);

            let mut req = Request::builder()
                .method(Method::POST)
                .uri("/")
                .header(CONTENT_TYPE, "application/json")
                .body(serialize_body(&payload))
                .unwrap();

            req.extensions_mut().insert(requester_id);

            let resp = app.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::GONE);

            let resp_body = deserialize_body::<ErrorResponse>(resp).await;
            let expected = ErrorResponse { error: String::from("Cannot reply to a deleted post") };
            assert_eq!(expected, resp_body);
        }
    }

    mod by_post_id {
        use super::*;

        #[tokio::test]
        async fn retrieves_the_post() {
            let [_, post, _] = post_with_author::all3();
            let post_clone = post.clone();

            let mut mock_pwa_read = MockPostWithAuthorRead::new();
            mock_pwa_read
                .expect_by_post_id()
                .with(eq(post.id))
                .once()
                .return_once(|_| Ok(post_clone));

            let state =
                AppState { post_with_author_read: Arc::new(mock_pwa_read), ..Default::default() };
            let app = super::routes().with_state(state);

            let req = Request::builder()
                .method(Method::GET)
                .uri(format!("/{}", post.id))
                .body(Body::empty())
                .unwrap();

            let resp = app.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);

            let resp_body = deserialize_body::<PostResponse>(resp).await;
            assert_eq!(PostResponse::from(post), resp_body);
        }

        #[tokio::test]
        async fn translates_errors() {
            let post_id = 2414;

            let mut mock_pwa_read = MockPostWithAuthorRead::new();
            mock_pwa_read
                .expect_by_post_id()
                .with(eq(post_id))
                .once()
                .return_once(|_| Err(ReadError::NotFound));

            let state =
                AppState { post_with_author_read: Arc::new(mock_pwa_read), ..Default::default() };
            let app = super::routes().with_state(state);

            let req = Request::builder()
                .method(Method::GET)
                .uri(format!("/{post_id}"))
                .body(Body::empty())
                .unwrap();

            let resp = app.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::NOT_FOUND);

            let resp_body = deserialize_body::<ErrorResponse>(resp).await;
            let expected = ErrorResponse { error: String::from("Not found") };
            assert_eq!(expected, resp_body);
        }
    }

    mod child_posts {
        use super::*;

        #[tokio::test]
        async fn retrieves_child_posts() {
            let parent_id = 92;
            let posts = post_with_author::all3(); // Not actually children
            let posts_vec = posts.to_vec();

            let mut mock_pwa_read = MockPostWithAuthorRead::new();
            mock_pwa_read
                .expect_by_parent()
                .with(eq(parent_id))
                .once()
                .return_once(move |_| Ok(posts_vec));

            let state =
                AppState { post_with_author_read: Arc::new(mock_pwa_read), ..Default::default() };
            let app = super::routes().with_state(state);

            let req = Request::builder()
                .method(Method::GET)
                .uri(format!("/{parent_id}/children"))
                .body(Body::empty())
                .unwrap();

            let resp = app.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);

            let resp_body = deserialize_body::<Vec<PostResponse>>(resp).await;
            assert_eq!(posts.map_into::<Vec<PostResponse>>(), resp_body);
        }

        #[tokio::test]
        async fn translates_errors() {
            let parent_id = 257;

            let mut mock_pwa_read = MockPostWithAuthorRead::new();
            mock_pwa_read
                .expect_by_parent()
                .with(eq(parent_id))
                .once()
                .return_once(move |_| Err(ReadError::Technical(anyhow!("bad things happened!"))));

            let state =
                AppState { post_with_author_read: Arc::new(mock_pwa_read), ..Default::default() };
            let app = super::routes().with_state(state);

            let req = Request::builder()
                .method(Method::GET)
                .uri(format!("/{parent_id}/children"))
                .body(Body::empty())
                .unwrap();

            let resp = app.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);

            let resp_body = deserialize_body::<ErrorResponse>(resp).await;
            let expected = ErrorResponse { error: String::from("internal server error") };
            assert_eq!(expected, resp_body);
        }
    }

    mod specific_user_posts {
        use super::*;

        #[tokio::test]
        async fn retrieves_posts_by_a_user() {
            let posts = post_with_author::all3();
            let posts_vec = posts.to_vec();
            let author_username = String::from("some_user");

            let mut mock_pwa_read = MockPostWithAuthorRead::new();
            mock_pwa_read
                .expect_by_author_username()
                .with(eq(author_username.clone()))
                .once()
                .return_once(|_| Ok(posts_vec));

            let state =
                AppState { post_with_author_read: Arc::new(mock_pwa_read), ..Default::default() };
            let app = super::routes().with_state(state);

            let req = Request::builder()
                .method(Method::GET)
                .uri(format!("/user/{author_username}"))
                .body(Body::empty())
                .unwrap();

            let resp = app.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);

            let resp_body = deserialize_body::<Vec<PostResponse>>(resp).await;
            assert_eq!(posts.map_into::<Vec<PostResponse>>(), resp_body);
        }

        #[tokio::test]
        async fn translates_errors() {
            let username = "anything_here";

            let mut mock_pwa_read = MockPostWithAuthorRead::new();
            mock_pwa_read
                .expect_by_author_username()
                .with(eq(username))
                .once()
                .return_once(|_| Err(ReadError::Technical(anyhow!("oh no!"))));

            let state =
                AppState { post_with_author_read: Arc::new(mock_pwa_read), ..Default::default() };
            let app = super::routes().with_state(state);

            let req = Request::builder()
                .method(Method::GET)
                .uri(format!("/user/{username}"))
                .body(Body::empty())
                .unwrap();

            let resp = app.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);

            let resp_body = deserialize_body::<ErrorResponse>(resp).await;
            let expected = ErrorResponse { error: String::from("internal server error") };
            assert_eq!(expected, resp_body);
        }
    }

    mod own_posts {
        use super::*;

        #[tokio::test]
        async fn retrieves_posts_by_the_requester() {
            let requester_id = 422;
            let posts = post_with_author::all3();
            let posts_vec = posts.to_vec();

            let mut mock_pwa_read = MockPostWithAuthorRead::new();
            mock_pwa_read
                .expect_by_author()
                .with(eq(requester_id))
                .once()
                .return_once(|_| Ok(posts_vec));

            let state =
                AppState { post_with_author_read: Arc::new(mock_pwa_read), ..Default::default() };
            let app = super::routes().with_state(state);

            let mut req = Request::builder()
                .method(Method::GET)
                .uri("/me")
                .body(Body::empty())
                .unwrap();

            req.extensions_mut().insert(requester_id);

            let resp = app.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);

            let resp_body = deserialize_body::<Vec<PostResponse>>(resp).await;
            assert_eq!(posts.map_into::<Vec<PostResponse>>(), resp_body);
        }

        #[tokio::test]
        async fn translates_errors() {
            let requester_id = 24;

            let mut mock_pwa_read = MockPostWithAuthorRead::new();
            mock_pwa_read
                .expect_by_author()
                .with(eq(requester_id))
                .once()
                .return_once(|_| Err(ReadError::NotFound));

            let state =
                AppState { post_with_author_read: Arc::new(mock_pwa_read), ..Default::default() };
            let app = super::routes().with_state(state);

            let mut req = Request::builder()
                .method(Method::GET)
                .uri("/me")
                .body(Body::empty())
                .unwrap();

            req.extensions_mut().insert(requester_id);

            let resp = app.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::NOT_FOUND);

            let resp_body = deserialize_body::<ErrorResponse>(resp).await;
            let expected = ErrorResponse { error: String::from("Not found") };
            assert_eq!(expected, resp_body);
        }
    }
}
