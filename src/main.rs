mod modules;
mod utils;

use axum::Router;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tower_http::LatencyUnit;
use tracing::Level;

use modules::{day_five, day_negative_one, day_nine, day_two};
use utils::rate_limit::RateLimit;

#[derive(Debug, Clone)]
pub struct AppState {
    pub rate_limit: RateLimit,
}

impl AppState {
    fn new() -> Self {
        Self {
            rate_limit: RateLimit::default(),
        }
    }
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(
            DefaultMakeSpan::new()
                .level(Level::INFO)
                .include_headers(true),
        )
        .on_response(
            DefaultOnResponse::new()
                .level(Level::INFO)
                .include_headers(true)
                .latency_unit(LatencyUnit::Micros),
        );
    tracing::info!("tracing is initialized");

    let router = Router::new()
        .nest("/", day_negative_one::routes())
        .nest("/2", day_two::routes())
        .nest("/5", day_five::routes())
        .nest("/9", day_nine::routes())
        .layer(trace_layer)
        .with_state(AppState::new());

    Ok(router.into())
}
