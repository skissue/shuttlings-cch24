use chrono::{DateTime, Utc};
use poem::{
    delete, get, handler,
    http::StatusCode,
    post, put,
    web::{Data, Path, Query},
    Body, EndpointExt as _, IntoEndpoint, Response, Route,
};
use rand::distributions::{Alphanumeric, DistString as _};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

pub fn route() -> impl IntoEndpoint {
    Route::new()
        .at("/reset", post(quotes_reset))
        .at("/draft", post(quotes_draft))
        .at("/cite/:id", get(quotes_cite))
        .at("/undo/:id", put(quotes_update))
        .at("/remove/:id", delete(quotes_delete))
        .at("/list", get(quotes_paginate))
        .data(PaginationStatuses(Arc::new(Mutex::new(HashMap::new()))))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Quote {
    id: Option<Uuid>,
    author: String,
    quote: String,
    version: Option<i32>,
    created_at: Option<DateTime<Utc>>,
}

#[handler]
async fn quotes_reset(pool: Data<&PgPool>) {
    sqlx::query!("DELETE FROM quotes")
        .execute(*pool)
        .await
        .expect("Clearing table shouldn't fail");
}

#[handler]
async fn quotes_draft(pool: Data<&PgPool>, body: Body) -> Response {
    let quote: Quote = body.into_json().await.unwrap();

    let inserted = sqlx::query_as!(
        Quote,
        "INSERT INTO quotes (id, author, quote) VALUES ($1, $2, $3) RETURNING *",
        Uuid::new_v4(),
        quote.author,
        quote.quote
    )
    .fetch_one(*pool)
    .await
    .unwrap();

    Response::builder()
        .status(StatusCode::CREATED)
        .body(serde_json::to_string(&inserted).unwrap())
}

#[handler]
async fn quotes_cite(pool: Data<&PgPool>, id: Path<Uuid>) -> Response {
    let result = sqlx::query_as!(Quote, "SELECT * FROM quotes WHERE id = $1", *id)
        .fetch_optional(*pool)
        .await
        .unwrap();
    let Some(quote) = result else {
        return StatusCode::NOT_FOUND.into();
    };

    serde_json::to_string(&quote).unwrap().into()
}

#[handler]
async fn quotes_update(pool: Data<&PgPool>, id: Path<Uuid>, body: Body) -> Response {
    let new_quote: Quote = body.into_json().await.unwrap();

    let result = sqlx::query_as!(
        Quote,
        "UPDATE quotes SET author = $2, quote = $3, version = version + 1 WHERE id = $1 RETURNING *",
        *id,
        new_quote.author,
        new_quote.quote
    )
    .fetch_optional(*pool)
    .await
    .unwrap();

    let Some(new_quote) = result else {
        return StatusCode::NOT_FOUND.into();
    };

    serde_json::to_string(&new_quote).unwrap().into()
}

#[handler]
async fn quotes_delete(pool: Data<&PgPool>, id: Path<Uuid>) -> Response {
    let result = sqlx::query_as!(Quote, "DELETE FROM quotes WHERE id = $1 RETURNING *", *id)
        .fetch_optional(*pool)
        .await
        .unwrap();
    let Some(quote) = result else {
        return StatusCode::NOT_FOUND.into();
    };

    serde_json::to_string(&quote).unwrap().into()
}

#[derive(Debug, Clone)]
struct PaginationStatus {
    page: usize,
    remaining: Vec<Quote>,
}

#[derive(Debug, Clone)]
struct PaginationStatuses(Arc<Mutex<HashMap<String, PaginationStatus>>>);

#[derive(Serialize)]
struct QuotePaginationResponse {
    page: usize,
    quotes: Vec<Quote>,
    next_token: Option<String>,
}

#[handler]
async fn quotes_paginate(
    pool: Data<&PgPool>,
    token: Query<HashMap<String, String>>,
    pagination_statuses: Data<&PaginationStatuses>,
) -> Response {
    if let Some(token) = token.get("token") {
        let mut lock = pagination_statuses.0 .0.lock().await;
        let Some(pagination_status) = lock.get(token).map(|s| s.clone()) else {
            return StatusCode::BAD_REQUEST.into();
        };
        let mut quotes = pagination_status.remaining;
        let mut next_token = None;

        if quotes.len() > 3 {
            next_token = Some(Alphanumeric.sample_string(&mut rand::thread_rng(), 16));

            let rest = quotes.split_off(3);
            lock.insert(
                next_token.as_ref().unwrap().clone(),
                PaginationStatus {
                    page: pagination_status.page + 1,
                    remaining: rest,
                },
            );
        }

        serde_json::to_string(&QuotePaginationResponse {
            page: pagination_status.page,
            quotes,
            next_token,
        })
        .unwrap()
        .into()
    } else {
        let mut quotes = sqlx::query_as!(Quote, "SELECT * FROM quotes ORDER BY created_at ASC")
            .fetch_all(*pool)
            .await
            .unwrap();
        let mut next_token = None;

        if quotes.len() > 3 {
            next_token = Some(Alphanumeric.sample_string(&mut rand::thread_rng(), 16));

            let rest = quotes.split_off(3);
            pagination_statuses.0 .0.lock().await.insert(
                next_token.as_ref().unwrap().clone(),
                PaginationStatus {
                    page: 2,
                    remaining: rest,
                },
            );
        }

        serde_json::to_string(&QuotePaginationResponse {
            page: 1,
            quotes,
            next_token,
        })
        .unwrap()
        .into()
    }
}
