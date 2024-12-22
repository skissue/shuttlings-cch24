use cargo_manifest::Manifest;
use itertools::Itertools;
use poem::{
    handler,
    http::{HeaderMap, StatusCode},
    post, Response, Route,
};
use std::str::FromStr as _;

pub fn route() -> Route {
    Route::new().at("/manifest", post(order_manifests))
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
