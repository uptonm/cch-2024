use axum::body::Body;
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, RouterIntoService};
use axum::Router;

pub fn routes() -> RouterIntoService<Body> {
    Router::new()
        .route("/", get(hello_bird))
        .route("/-1/seek", get(seek))
        .into_service()
}

#[tracing::instrument]
async fn hello_bird() -> &'static str {
    "Hello, bird!"
}

const REDIRECT_URL: &str = "https://www.youtube.com/watch?v=9Gc4QTqslN4";

#[tracing::instrument]
async fn seek() -> impl IntoResponse {
    (StatusCode::FOUND, [(header::LOCATION, REDIRECT_URL)])
}
