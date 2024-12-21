use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::header::CONTENT_TYPE;
use axum::http::StatusCode;
use axum::response::Response;
use axum::routing::{delete, get, post, put, RouterIntoService};
use axum::{Json, Router};
use serde::Deserialize;
use shuttle_persist::PersistInstance;
use sqlx::types::Uuid;

use crate::utils::error_handling::Result;
use crate::utils::quote::{ListResponse, QuotePayload, QuoteState};

pub fn routes(pool: sqlx::PgPool, persist: PersistInstance) -> RouterIntoService<Body> {
    Router::new()
        .route("/reset", post(reset))
        .route("/cite/:id", get(cite))
        .route("/remove/:id", delete(remove))
        .route("/undo/:id", put(undo))
        .route("/draft", post(draft))
        .route("/list", get(list))
        .with_state(QuoteState::new(pool, persist))
        .into_service()
}

async fn reset(State(state): State<QuoteState>) -> StatusCode {
    match state.reset().await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn cite(State(state): State<QuoteState>, Path(id): Path<Uuid>) -> Result<Response> {
    let quote = state.get_quote(id).await?;

    let Some(quote) = quote else {
        return Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())?);
    };

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(serde_json::to_string_pretty(&quote)?))?)
}

async fn remove(State(state): State<QuoteState>, Path(id): Path<Uuid>) -> Result<Response> {
    let quote = state.delete_quote(id).await?;

    let Some(quote) = quote else {
        return Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())?);
    };

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(serde_json::to_string_pretty(&quote)?))?)
}

async fn undo(
    State(state): State<QuoteState>,
    Path(id): Path<Uuid>,
    Json(quote): Json<QuotePayload>,
) -> Result<Response> {
    let quote = state.update_quote(id, quote).await?;

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
    State(state): State<QuoteState>,
    Json(quote): Json<QuotePayload>,
) -> Result<Response> {
    let quote = state.create_quote(quote).await?;

    Ok(Response::builder()
        .status(StatusCode::CREATED)
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string_pretty(&quote)?))?)
}

const PAGE_SIZE: i32 = 3;

#[derive(Debug, Deserialize)]
struct ListQuery {
    token: Option<String>,
}

async fn list(Query(query): Query<ListQuery>, State(state): State<QuoteState>) -> Result<Response> {
    let mut current_page = 1;
    if let Some(token) = query.token {
        let Ok(Some(page_token)) = state.get_next_page_token(token) else {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::empty())?);
        };

        current_page = page_token;
    }

    let current_offset = (current_page - 1) * PAGE_SIZE;

    let quotes = state.list_quotes(PAGE_SIZE + 1, current_offset).await?;

    let mut next_token = None;
    if quotes.len() as i32 > PAGE_SIZE {
        next_token = Some(state.create_next_page_token(current_page + 1)?);
    }

    let payload = ListResponse::new(quotes, current_page, next_token);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(serde_json::to_string_pretty(&payload)?))?)
}
