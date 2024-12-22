mod connect4;

use connect4::{Connect4, MoveError, Tile};
use poem::{
    get, handler,
    http::StatusCode,
    post,
    web::{Data, Path},
    EndpointExt as _, IntoEndpoint, Response, Route,
};
use rand::SeedableRng as _;
use std::sync::Arc;
use tokio::sync::RwLock;

pub fn route() -> impl IntoEndpoint {
    Route::new()
        .at("/board", get(get_connect4_board))
        .at("/reset", post(reset_connect4_board))
        .at("/place/:team/:column", post(play_connect4))
        .at("/random-board", get(get_random_connect4))
        .data(Arc::new(RwLock::new(Connect4::empty())))
        .data(Connect4Rng(Arc::new(RwLock::new(
            rand::rngs::StdRng::seed_from_u64(2024),
        ))))
}

#[derive(Clone)]
struct Connect4Rng(Arc<RwLock<rand::rngs::StdRng>>);

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

#[handler]
async fn get_random_connect4(rng: Data<&Connect4Rng>) -> String {
    format!("{}", Connect4::random(&mut *rng.0 .0.write().await))
}
