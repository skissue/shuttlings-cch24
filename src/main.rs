use cargo_manifest::Manifest;
use itertools::Itertools;
use poem::{
    get, handler,
    http::{header, HeaderMap, StatusCode},
    post,
    web::{
        headers::{ContentType, Header},
        Data, Query, TypedHeader,
    },
    EndpointExt, Response, Route,
};
use serde::{Deserialize, Serialize};
use shuttle_poem::ShuttlePoem;
use std::{
    cell::RefCell,
    net::{Ipv4Addr, Ipv6Addr},
    sync::{Arc, RwLock},
    time::Duration,
};
use tokio::sync::Mutex;
use toml::Table;

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
fn order_manifests(body: Vec<u8>) -> Response {
    let Ok(manifest) = Manifest::from_slice(&body) else {
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

#[shuttle_runtime::main]
async fn poem() -> ShuttlePoem<impl poem::Endpoint> {
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
        .data(MilkBucket(Arc::new(Mutex::new(
            leaky_bucket::RateLimiter::builder()
                .initial(5)
                .max(5)
                .interval(Duration::from_secs(1))
                .build(),
        ))));

    Ok(app.into())
}
