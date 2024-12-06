use std::net::Ipv4Addr;

use poem::{
    get, handler,
    http::{header, StatusCode},
    web::Query,
    Response, Route,
};
use serde::Deserialize;
use shuttle_poem::ShuttlePoem;

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

#[shuttle_runtime::main]
async fn poem() -> ShuttlePoem<impl poem::Endpoint> {
    let app = Route::new()
        .at("/", get(hello_bird))
        .at("/-1/seek", get(seek))
        .at("/2/dest", get(encrypt_address))
        .at("/2/key", get(get_address_key));

    Ok(app.into())
}
