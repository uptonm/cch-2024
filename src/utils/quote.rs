use std::{ops::Deref, sync::Arc};

use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use shuttle_persist::PersistInstance;
use sqlx::{
    types::{
        chrono::{DateTime, Utc},
        Uuid,
    },
    FromRow,
};

use crate::utils::error_handling::Result;

#[derive(Clone)]
pub struct QuoteStateInternal {
    pool: sqlx::PgPool,
    persist: PersistInstance,
}

#[derive(Clone)]
pub struct QuoteState(Arc<QuoteStateInternal>);

impl Deref for QuoteState {
    type Target = QuoteStateInternal;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl QuoteState {
    pub fn new(pool: sqlx::PgPool, persist: PersistInstance) -> Self {
        Self(Arc::new(QuoteStateInternal { pool, persist }))
    }

    pub async fn reset(&self) -> Result<()> {
        sqlx::query("DELETE FROM quotes")
            .execute(&self.pool)
            .await?;
        self.persist.clear()?;
        Ok(())
    }

    pub async fn get_quote(&self, id: Uuid) -> Result<Option<Quote>> {
        let quote = sqlx::query_as::<_, Quote>("SELECT * FROM quotes WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(quote)
    }

    pub async fn delete_quote(&self, id: Uuid) -> Result<Option<Quote>> {
        let quote = sqlx::query_as::<_, Quote>("DELETE FROM quotes WHERE id = $1 RETURNING *")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(quote)
    }

    pub async fn update_quote(&self, id: Uuid, quote: QuotePayload) -> Result<Option<Quote>> {
        let quote = sqlx::query_as::<_, Quote>("UPDATE quotes SET author = $1, quote = $2, version = version + 1 WHERE id = $3 RETURNING *")
          .bind(quote.author)
          .bind(quote.quote)
          .bind(id)
          .fetch_optional(&self.pool)
          .await?;
        Ok(quote)
    }

    pub async fn create_quote(&self, quote: QuotePayload) -> Result<Quote> {
        let quote = sqlx::query_as::<_, Quote>(
            "INSERT INTO quotes (author, quote) VALUES ($1, $2) RETURNING *",
        )
        .bind(quote.author)
        .bind(quote.quote)
        .fetch_one(&self.pool)
        .await?;
        Ok(quote)
    }

    pub async fn list_quotes(&self, limit: i32, offset: i32) -> Result<Vec<Quote>> {
        let quotes = sqlx::query_as::<_, Quote>(
            "SELECT * FROM quotes ORDER BY created_at ASC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        Ok(quotes)
    }

    pub fn get_next_page_token(&self, token: String) -> Result<Option<i32>> {
        let Ok(page) = self.persist.load::<i32>(&token) else {
            return Ok(None);
        };
        // tokens are only one-time use
        self.persist.remove(&token)?;
        Ok(Some(page))
    }

    pub fn create_next_page_token(&self, page: i32) -> Result<String> {
        let token = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect::<String>();
        self.persist.save(&token, page)?;
        Ok(token)
    }
}

#[derive(Debug, Deserialize, Serialize, FromRow)]
pub struct Quote {
    id: Uuid,
    author: String,
    quote: String,
    created_at: DateTime<Utc>,
    version: i32,
}

#[derive(Debug, Deserialize)]
pub struct QuotePayload {
    author: String,
    quote: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ListResponse {
    quotes: Vec<Quote>,
    page: i32,
    next_token: Option<String>,
}

impl ListResponse {
    pub fn new(quotes: Vec<Quote>, page: i32, next_token: Option<String>) -> Self {
        Self {
            quotes,
            page,
            next_token,
        }
    }
}
