use poem::{
    get, handler,
    http::{header, StatusCode},
    Response, Route,
};
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

#[shuttle_runtime::main]
async fn poem() -> ShuttlePoem<impl poem::Endpoint> {
    let app = Route::new()
        .at("/", get(hello_bird))
        .at("/-1/seek", get(seek));

    Ok(app.into())
}
