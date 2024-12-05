use poem::{get, handler, Route};
use shuttle_poem::ShuttlePoem;

#[handler]
fn hello_bird() -> &'static str {
    "Hello, bird!"
}

#[shuttle_runtime::main]
async fn poem() -> ShuttlePoem<impl poem::Endpoint> {
    let app = Route::new().at("/", get(hello_bird));

    Ok(app.into())
}
