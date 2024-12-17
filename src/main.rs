mod modules;
mod utils;

use axum::Router;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tower_http::LatencyUnit;
use tracing::Level;

use modules::{day_five, day_negative_one, day_nine, day_twelve, day_two};

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
        .nest_service("/", day_negative_one::routes())
        .nest_service("/2", day_two::routes())
        .nest_service("/5", day_five::routes())
        .nest_service("/9", day_nine::routes())
        .nest_service("/12", day_twelve::routes())
        .layer(trace_layer);

    Ok(router.into())
}
