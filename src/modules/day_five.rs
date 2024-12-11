use axum::http::StatusCode;
use axum::response::Response;
use axum::routing::post;
use axum::Router;

use crate::utils::cargo_manifest::Metadata;
use crate::utils::error_responses::no_content;
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/manifest", post(manifest))
}

async fn manifest(metadata: Metadata) -> Response {
    if metadata.orders.is_empty() {
        return no_content();
    }

    Response::builder()
        .status(StatusCode::OK)
        .body(metadata.to_string().into())
        .unwrap()
}
