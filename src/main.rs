mod day0;
mod day12;
mod day16;
mod day19;
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
        .nest("/16", day16::route())
        .nest("/19", day19::route())
        .data(pool);

    Ok(app.into())
}
