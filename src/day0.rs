//! Called day0 instead of day-1 because the latter isn't a valid identifier.

use poem::{
    get, handler,
    http::{header, StatusCode}, Response, Route,
};

pub fn route() -> Route {
    Route::new()
        .at("/", get(hello_bird))
        .at("/-1/seek", get(seek))
}

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

