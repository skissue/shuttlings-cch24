mod connect4;

use cargo_manifest::Manifest;
use chrono::{DateTime, Utc};
use connect4::{Connect4, MoveError, Tile};
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
fn hello_bird() -> &'static str {
    "Hello, bird!"
}

#[handler]
fn seek() -> Response {
    Response::builder()
        .status(StatusCode::FOUND)
        .header(
            header::LOCATION,
            "https://www.youtube.com/watch?v=9Gc4QTqslN4",
        )
        .body(())
}

#[derive(Deserialize)]
struct EncryptParams {
    from: Ipv4Addr,
    key: Ipv4Addr,
}

#[derive(Deserialize)]
struct KeyParams {
    from: Ipv4Addr,
    to: Ipv4Addr,
}

#[derive(Deserialize)]
struct EncryptParamsV6 {
    from: Ipv6Addr,
    key: Ipv6Addr,
}

#[derive(Deserialize)]
struct KeyParamsV6 {
    from: Ipv6Addr,
    to: Ipv6Addr,
}

#[handler]
fn encrypt_address(params: Query<EncryptParams>) -> String {
    let Query(EncryptParams { from, key }) = params;

    let added: Vec<u8> = from
        .octets()
        .into_iter()
        .zip(key.octets().into_iter())
        .map(|(a, b)| a.wrapping_add(b))
        .collect();
    let dest = Ipv4Addr::new(added[0], added[1], added[2], added[3]);

    dest.to_string()
}

#[handler]
fn get_address_key(params: Query<KeyParams>) -> String {
    let Query(KeyParams { from, to }) = params;

    let diffed: Vec<u8> = from
        .octets()
        .into_iter()
        .zip(to.octets().into_iter())
        .map(|(a, b)| b.wrapping_sub(a))
        .collect();
    let key = Ipv4Addr::new(diffed[0], diffed[1], diffed[2], diffed[3]);

    key.to_string()
}

#[handler]
fn encrypt_address_ipv6(params: Query<EncryptParamsV6>) -> String {
    let Query(EncryptParamsV6 { from, key }) = params;

    let xored: Vec<u8> = from
        .octets()
        .into_iter()
        .zip(key.octets().into_iter())
        .map(|(a, b)| a ^ b)
        .collect();

    let mut octets: [u8; 16] = xored.try_into().unwrap();

    let to = Ipv6Addr::from(octets);

    to.to_string()
}

#[handler]
fn get_address_key_ipv6(params: Query<KeyParamsV6>) -> String {
    let Query(KeyParamsV6 { from, to }) = params;

    let xored: Vec<u8> = from
        .octets()
        .into_iter()
        .zip(to.octets().into_iter())
        .map(|(a, b)| a ^ b)
        .collect();

    let mut octets: [u8; 16] = xored.try_into().unwrap();

    let key = Ipv6Addr::from(octets);

    key.to_string()
}

#[handler]
fn order_manifests(headers: &HeaderMap, body: String) -> Response {
    // This is a horrendous shortcut, but hey, it works, and if it ain't broke,
    // don't fix it ¯\_(ツ)_/¯.
    let Some(data) = (match headers.get("Content-Type").and_then(|v| v.to_str().ok()) {
        Some("application/toml") => Some(body),
        Some("application/json") => serde_json::from_str(&body)
            .ok()
            .and_then(|v: serde_json::Value| toml::ser::to_string(&v).ok()),
        Some("application/yaml") => serde_yaml::from_str(&body)
            .ok()
            .and_then(|v: serde_json::Value| toml::ser::to_string(&v).ok()),
        _ => None,
    }) else {
        return StatusCode::UNSUPPORTED_MEDIA_TYPE.into();
    };

    let Ok(manifest) = Manifest::from_str(&data) else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("Invalid manifest");
    };

    let Some(package) = manifest.package else {
        return StatusCode::NO_CONTENT.into();
    };
    if package
        .keywords
        .filter(|k| {
            k.as_ref()
                .as_local()
                .unwrap()
                .contains(&"Christmas 2024".to_owned())
        })
        .is_none()
    {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("Magic keyword not provided");
    };

    let Some(orders) = package
        .metadata
        .and_then(|m| Some(m.get("orders")?.as_array()?.to_owned()))
    else {
        return StatusCode::NO_CONTENT.into();
    };

    let items: String = orders
        .into_iter()
        .filter_map(|map| {
            let item = map.get("item")?.as_str()?;
            let quantity = map.get("quantity")?.as_integer()?;

            Some(format!("{}: {}", item, quantity))
        })
        .intersperse("\n".to_owned())
        .collect();

    if items.is_empty() {
        return StatusCode::NO_CONTENT.into();
    }

    items.into()
}

#[derive(Clone)]
struct MilkBucket(Arc<Mutex<leaky_bucket::RateLimiter>>);

#[derive(Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
enum MilkConversion {
    Liters { liters: f32 },
    Gallons { gallons: f32 },
    IHateTheBritish { litres: f32 },
    IHateTheBritishMore { pints: f32 },
}

#[handler]
async fn leaky_milk(headers: &HeaderMap, bucket: Data<&MilkBucket>, body: String) -> Response {
    if !bucket.0 .0.lock().await.try_acquire(1) {
        return Response::builder()
            .status(StatusCode::TOO_MANY_REQUESTS)
            .body("No milk available\n");
    }

    if headers
        .get("Content-Type")
        .map_or(false, |v| v == "application/json")
    {
        let Ok(conversion_request): Result<MilkConversion, _> = serde_json::from_str(&body) else {
            return StatusCode::BAD_REQUEST.into();
        };

        let response = match conversion_request {
            MilkConversion::Liters { liters } => {
                let gallons = liters * 0.2641720524;
                MilkConversion::Gallons { gallons }
            }
            MilkConversion::Gallons { gallons } => {
                let liters = gallons * 3.785411784;
                MilkConversion::Liters { liters }
            }
            MilkConversion::IHateTheBritish { litres } => {
                let pints = litres * 1.75975;
                MilkConversion::IHateTheBritishMore { pints }
            }
            MilkConversion::IHateTheBritishMore { pints } => {
                let litres = pints * 0.56826;
                MilkConversion::IHateTheBritish { litres }
            }
        };
        return serde_json::to_string(&response).unwrap().into();
    }

    "Milk withdrawn\n".into()
}

#[handler]
async fn fill_milk_bucket(bucket: Data<&MilkBucket>) {
    *bucket.0 .0.lock().await = leaky_bucket::RateLimiter::builder()
        .initial(5)
        .max(5)
        .interval(Duration::from_secs(1))
        .build();
}

#[handler]
async fn get_connect4_board(board: Data<&Arc<RwLock<Connect4>>>) -> String {
    format!("{}", board.0.read().await)
}

#[handler]
async fn reset_connect4_board(
    board: Data<&Arc<RwLock<Connect4>>>,
    rng: Data<&Connect4Rng>,
) -> String {
    *board.0.write().await = Connect4::empty();
    *rng.0 .0.write().await = rand::rngs::StdRng::seed_from_u64(2024);
    format!("{}", board.0.read().await)
}

#[handler]
async fn play_connect4(
    board: Data<&Arc<RwLock<Connect4>>>,
    Path((team, column)): Path<(String, String)>,
) -> Response {
    let team = match team.as_str() {
        "cookie" => Tile::Cookie,
        "milk" => Tile::Milk,
        _ => return StatusCode::BAD_REQUEST.into(),
    };
    let Ok(column) = column.parse() else {
        return StatusCode::BAD_REQUEST.into();
    };

    let mut board = board.0.write().await;
    match board.play(team, column) {
        Err(MoveError::InvalidColumn) => StatusCode::BAD_REQUEST.into(),
        Err(MoveError::ColumnFull) | Err(MoveError::GameOver) => Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .body(format!("{}", board)),
        _ => format!("{}", board).into(),
    }
}

#[derive(Clone)]
struct Connect4Rng(Arc<RwLock<rand::rngs::StdRng>>);

#[handler]
async fn get_random_connect4(rng: Data<&Connect4Rng>) -> String {
    format!("{}", Connect4::random(&mut *rng.0 .0.write().await))
}

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
        .at("/", get(hello_bird))
        .at("/-1/seek", get(seek))
        .at("/2/dest", get(encrypt_address))
        .at("/2/key", get(get_address_key))
        .at("/2/v6/dest", get(encrypt_address_ipv6))
        .at("/2/v6/key", get(get_address_key_ipv6))
        .at("/5/manifest", post(order_manifests))
        .at("/9/milk", post(leaky_milk))
        .at("/9/refill", post(fill_milk_bucket))
        .at("/12/board", get(get_connect4_board))
        .at("/12/reset", post(reset_connect4_board))
        .at("/12/place/:team/:column", post(play_connect4))
        .at("/12/random-board", get(get_random_connect4))
        .at("/16/wrap", post(wrap_gift))
        .at("/16/unwrap", get(unwrap_gift))
        .at("/16/decode", post(decode_old_gift))
        .at("/19/reset", post(quotes_reset))
        .at("/19/draft", post(quotes_draft))
        .at("/19/cite/:id", get(quotes_cite))
        .at("/19/undo/:id", put(quotes_update))
        .at("/19/remove/:id", delete(quotes_delete))
        .at("/19/list", get(quotes_paginate))
        .data(MilkBucket(Arc::new(Mutex::new(
            leaky_bucket::RateLimiter::builder()
                .initial(5)
                .max(5)
                .interval(Duration::from_secs(1))
                .build(),
        ))))
        .data(Arc::new(RwLock::new(Connect4::empty())))
        .data(Connect4Rng(Arc::new(RwLock::new(
            rand::rngs::StdRng::seed_from_u64(2024),
        ))))
        .data(pool)
        .data(PaginationStatuses(Arc::new(Mutex::new(HashMap::new()))));

    Ok(app.into())
}
