use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::header::CONTENT_TYPE;
use axum::http::StatusCode;
use axum::response::Response;
use axum::routing::{delete, get, post, put, RouterIntoService};
use axum::{Json, Router};
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::types::Uuid;

use crate::utils::error_handling::Result;

pub fn routes(pool: sqlx::PgPool) -> RouterIntoService<Body> {
    Router::new()
        .route("/reset", post(reset))
        .route("/cite/:id", get(cite))
        .route("/remove/:id", delete(remove))
        .route("/undo/:id", put(undo))
        .route("/draft", post(draft))
        .route("/list", get(list))
        .with_state(Arc::new(pool))
        .into_service()
}

async fn reset(State(pool): State<Arc<sqlx::PgPool>>) -> StatusCode {
    let Ok(_) = sqlx::query("DELETE FROM quotes").execute(&*pool).await else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };
    StatusCode::OK
}

#[derive(Debug, Deserialize, Serialize, FromRow)]
struct Quote {
    id: Uuid,
    author: String,
    quote: String,
    created_at: DateTime<Utc>,
    version: i32,
}

async fn cite(State(pool): State<Arc<sqlx::PgPool>>, Path(id): Path<Uuid>) -> Result<Response> {
    let quote = sqlx::query_as::<_, Quote>("SELECT * FROM quotes WHERE id = $1")
        .bind(id)
        .fetch_optional(&*pool)
        .await?;

    let Some(quote) = quote else {
        return Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())?);
    };

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(serde_json::to_string_pretty(&quote)?))?)
}

async fn remove(State(pool): State<Arc<sqlx::PgPool>>, Path(id): Path<Uuid>) -> Result<Response> {
    let quote = sqlx::query_as::<_, Quote>("DELETE FROM quotes WHERE id = $1 RETURNING *")
        .bind(id)
        .fetch_optional(&*pool)
        .await?;

    let Some(quote) = quote else {
        return Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())?);
    };

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(serde_json::to_string_pretty(&quote)?))?)
}

#[derive(Debug, Deserialize)]
struct QuotePayload {
    author: String,
    quote: String,
}

async fn undo(
    State(pool): State<Arc<sqlx::PgPool>>,
    Path(id): Path<Uuid>,
    Json(quote): Json<QuotePayload>,
) -> Result<Response> {
    let quote = sqlx::query_as::<_, Quote>(
        "UPDATE quotes SET author = $1, quote = $2, version = version + 1 WHERE id = $3 RETURNING *",
    )
    .bind(quote.author)
    .bind(quote.quote)
    .bind(id)
    .fetch_optional(&*pool)
    .await?;

    let Some(quote) = quote else {
        return Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())?);
    };

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string_pretty(&quote)?))?)
}

async fn draft(
    State(pool): State<Arc<sqlx::PgPool>>,
    Json(quote): Json<QuotePayload>,
) -> Result<Response> {
    let quote = sqlx::query_as::<_, Quote>(
        "INSERT INTO quotes (author, quote) VALUES ($1, $2) RETURNING *",
    )
    .bind(quote.author)
    .bind(quote.quote)
    .fetch_one(&*pool)
    .await?;

    Ok(Response::builder()
        .status(StatusCode::CREATED)
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string_pretty(&quote)?))?)
}

const PAGE_SIZE: i32 = 3;

#[derive(Debug, Deserialize, Serialize)]
struct ListResponse {
    quotes: Vec<Quote>,
    page: i32,
    next_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ListQuery {
    token: Option<String>,
}

async fn list(
    Query(ListQuery { token }): Query<ListQuery>,
    State(pool): State<Arc<sqlx::PgPool>>,
) -> Result<Response> {
    let mut current_page = 1;
    if let Some(token) = token {
        let Ok(page_token) =
            sqlx::query_scalar::<_, i32>("SELECT next_page FROM page_tokens WHERE token = $1")
                .bind(token)
                .fetch_one(&*pool)
                .await
        else {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::empty())?);
        };

        current_page = page_token;
    }

    let quote_count = sqlx::query_scalar::<_, i32>("SELECT COUNT(*)::INT FROM quotes")
        .fetch_one(&*pool)
        .await?;

    let quotes = sqlx::query_as::<_, Quote>(
        "SELECT * FROM quotes ORDER BY created_at ASC LIMIT $1 OFFSET $2",
    )
    .bind(PAGE_SIZE)
    .bind((current_page - 1) * PAGE_SIZE)
    .fetch_all(&*pool)
    .await?;

    let next_token = if quote_count > (current_page - 1) * PAGE_SIZE + PAGE_SIZE {
        let token = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect();
        sqlx::query("INSERT INTO page_tokens (token, next_page) VALUES ($1, $2)")
            .bind(&token)
            .bind(current_page + 1)
            .execute(&*pool)
            .await?;
        Some(token)
    } else {
        None
    };

    let payload = ListResponse {
        quotes,
        page: current_page,
        next_token,
    };

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(serde_json::to_string_pretty(&payload)?))?)
}
