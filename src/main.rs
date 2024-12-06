use cargo_manifest::Manifest;
use itertools::Itertools;
use poem::{
    get, handler,
    http::{header, StatusCode},
    post,
    web::Query,
    Response, Route,
};
use serde::Deserialize;
use shuttle_poem::ShuttlePoem;
use std::net::{Ipv4Addr, Ipv6Addr};
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
    let Some(metadata) = package.metadata else {
        return StatusCode::NO_CONTENT.into();
    };

    let orders = metadata["orders"].as_array().unwrap();

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

#[shuttle_runtime::main]
async fn poem() -> ShuttlePoem<impl poem::Endpoint> {
    let app = Route::new()
        .at("/", get(hello_bird))
        .at("/-1/seek", get(seek))
        .at("/2/dest", get(encrypt_address))
        .at("/2/key", get(get_address_key))
        .at("/2/v6/dest", get(encrypt_address_ipv6))
        .at("/2/v6/key", get(get_address_key_ipv6))
        .at("/5/manifest", post(order_manifests));

    Ok(app.into())
}
