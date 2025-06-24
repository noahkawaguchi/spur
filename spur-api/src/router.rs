use crate::{
    config::AppState,
    handler::{auth, content, friendship, post, prompt},
    middleware::validate_jwt,
};
use axum::{Router, http::StatusCode, middleware, routing::get};

pub fn create(state: AppState) -> Router {
    Router::new()
        .nest("/auth", auth::routes().with_state(state.clone())) // The only public routes
        .merge(protected_routes(state))
}

fn protected_routes(state: AppState) -> Router {
    Router::new()
        .route("/auth/check", get(|| async { StatusCode::NO_CONTENT })) // Simple token check route
        .nest("/friends", friendship::routes())
        .nest("/prompts", prompt::routes())
        .nest("/posts", post::routes())
        .nest("/content", content::routes())
        .route_layer(middleware::from_fn_with_state(state.clone(), validate_jwt))
        .with_state(state)
}
