use std::ops::DerefMut;

use axum::body::Body;
use axum::extract::State;
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use axum::routing::{post, RouterIntoService};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::utils::error_handling::Result;
use crate::utils::rate_limit::{filled_bucket, RateLimit};

pub fn routes() -> RouterIntoService<Body> {
    Router::new()
        .route("/milk", post(milk))
        .route("/refill", post(refill))
        .with_state(RateLimit::default())
        .into_service()
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum MilkPayload {
    Gallons(f32),
    Liters(f32),
    Litres(f32),
    Pints(f32),
}

impl From<MilkPayload> for Body {
    fn from(payload: MilkPayload) -> Self {
        Body::from(serde_json::to_string(&payload).unwrap())
    }
}

#[allow(clippy::excessive_precision)]
impl MilkPayload {
    fn convert(&self) -> MilkPayload {
        match self {
            Self::Liters(n) => Self::Gallons(0.264172060 * n),
            Self::Gallons(n) => Self::Liters(3.78541 * n),
            Self::Litres(n) => Self::Pints(1.759754 * n),
            Self::Pints(n) => Self::Litres(0.56826125 * n),
        }
    }
}

async fn milk(
    State(rate_limit): State<RateLimit>,
    headers: HeaderMap,
    payload: Option<Json<MilkPayload>>,
) -> Result<Response> {
    let has_milk = rate_limit.lock().await.try_acquire(1);
    if !has_milk {
        return too_many_requests();
    }

    let Some(content_type) = headers.get(CONTENT_TYPE) else {
        return milk_withdrawn();
    };

    if content_type != "application/json" {
        return milk_withdrawn();
    }

    let Some(Json(payload)) = payload else {
        return bad_request();
    };

    converted_milk(payload.convert())
}

async fn refill(State(rate_limit): State<RateLimit>) -> Result<Response> {
    let mut lock = rate_limit.lock().await;
    let bucket = lock.deref_mut();
    *bucket = filled_bucket();
    ok()
}

fn too_many_requests() -> Result<Response> {
    Ok(Response::builder()
        .status(StatusCode::TOO_MANY_REQUESTS)
        .body("No milk available\n".into())?)
}

fn milk_withdrawn() -> Result<Response> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .body("Milk withdrawn\n".into())?)
}

fn bad_request() -> Result<Response> {
    Ok(Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::empty())?)
}

fn converted_milk(payload: MilkPayload) -> Result<Response> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(payload.into())?)
}

fn ok() -> Result<Response> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())?)
}
