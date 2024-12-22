use std::net::{Ipv4Addr, Ipv6Addr};

use poem::{get, handler, web::Query, Route};
use serde::Deserialize;

pub fn route() -> Route {
    Route::new()
        .at("/dest", get(encrypt_address))
        .at("/key", get(get_address_key))
        .at("/v6/dest", get(encrypt_address_ipv6))
        .at("/v6/key", get(get_address_key_ipv6))
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
