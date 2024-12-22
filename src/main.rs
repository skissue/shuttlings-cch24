mod day0;
mod day12;
mod day2;
mod day5;
mod day9;

use cargo_manifest::Manifest;
use chrono::{DateTime, Utc};
use itertools::Itertools;
use jsonwebtoken::{errors::ErrorKind as JWTError, DecodingKey, EncodingKey, Header, Validation};
use poem::{
    delete, get, handler,
    http::{header, HeaderMap, StatusCode},
    post, put,
    web::{Data, Path, Query},
    Body, EndpointExt, Response, Route,
};
use rand::{
    distributions::{Alphanumeric, DistString},
    SeedableRng,
};
use serde::{Deserialize, Serialize};
use shuttle_poem::ShuttlePoem;
use sqlx::PgPool;
use std::{
    collections::{HashMap, HashSet},
    net::{Ipv4Addr, Ipv6Addr},
    str::FromStr,
    sync::Arc,
    time::Duration,
};
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

#[handler]
async fn wrap_gift(body: String) -> Response {
    let jwt = jsonwebtoken::encode(
        &Header::default(),
        &serde_json::from_str::<serde_json::Value>(&body).expect("Input is valid JSON"),
        &EncodingKey::from_secret(b"a"),
    )
    .expect("Failed to encode JWT");

    Response::builder()
        .header("Set-Cookie", format!("gift={jwt}",))
        .body(())
}

#[handler]
async fn unwrap_gift(headers: &HeaderMap) -> Response {
    let Some(cookie) = headers
        .get("Cookie")
        .and_then(|c| c.to_str().ok())
        .and_then(|c| c.strip_prefix("gift="))
    else {
        return StatusCode::BAD_REQUEST.into();
    };

    let mut validation = Validation::default();
    validation.required_spec_claims = HashSet::new();
    validation.validate_exp = false;

    let Ok(decoded) = jsonwebtoken::decode::<serde_json::Value>(
        cookie,
        &DecodingKey::from_secret(b"a"),
        &validation,
    )
    .map(|d| d.claims) else {
        return StatusCode::BAD_REQUEST.into();
    };

    decoded.to_string().into()
}

#[handler]
async fn decode_old_gift(body: String) -> Response {
    let mut validation = Validation::default();
    validation.required_spec_claims = HashSet::new();
    validation.validate_exp = false;
    validation.algorithms = vec![
        jsonwebtoken::Algorithm::RS256,
        jsonwebtoken::Algorithm::RS512,
    ];

    let decoded = match jsonwebtoken::decode::<serde_json::Value>(
        &body,
        &DecodingKey::from_rsa_pem(include_bytes!("../day16_santa_public_key.pem"))
            .expect("Key from filesystem is valid"),
        &validation,
    )
    .map(|c| c.claims)
    {
        Ok(decoded) => decoded,
        Err(err) => {
            return (match err.into_kind() {
                JWTError::InvalidSignature => StatusCode::UNAUTHORIZED,
                _ => StatusCode::BAD_REQUEST,
            })
            .into()
        }
    };

    decoded.to_string().into()
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

#[shuttle_runtime::main]
async fn poem(
    #[shuttle_shared_db::Postgres(local_uri = "postgres://localhost:5432/")] pool: sqlx::PgPool,
) -> ShuttlePoem<impl poem::Endpoint> {
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let app = Route::new()
        .nest_no_strip("/", day0::route())
        .nest("/2", day2::route())
        .nest("/5", day5::route())
        .nest("/9", day9::route())
        .nest("/12", day12::route())
        .at("/16/wrap", post(wrap_gift))
        .at("/16/unwrap", get(unwrap_gift))
        .at("/16/decode", post(decode_old_gift))
        .at("/19/reset", post(quotes_reset))
        .at("/19/draft", post(quotes_draft))
        .at("/19/cite/:id", get(quotes_cite))
        .at("/19/undo/:id", put(quotes_update))
        .at("/19/remove/:id", delete(quotes_delete))
        .at("/19/list", get(quotes_paginate))
        .data(pool)
        .data(PaginationStatuses(Arc::new(Mutex::new(HashMap::new()))));

    Ok(app.into())
}
