use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use tower_http::trace::TraceLayer;

#[tracing::instrument]
async fn hello_bird() -> &'static str {
    "Hello, bird!"
}

const REDIRECT_URL: &str = "https://www.youtube.com/watch?v=9Gc4QTqslN4";

#[tracing::instrument]
async fn seek() -> impl IntoResponse {
    (StatusCode::FOUND, [(header::LOCATION, REDIRECT_URL)])
}

pub fn routes() -> Router {
    Router::new()
        .layer(TraceLayer::new_for_http())
        .route("/", get(hello_bird))
        .route("/-1/seek", get(seek))
}
