use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Response;
use axum::routing::{get, post, RouterIntoService};
use axum::Router;
use rand::rngs::StdRng;
use rand::SeedableRng;
use tokio::sync::RwLock;

use crate::utils::connect_four::{Connect4, Player, BOARD_SIZE};
use crate::utils::error_handling::Result;

pub fn routes() -> RouterIntoService<Body> {
    Router::new()
        .route("/board", get(board))
        .route("/reset", post(reset))
        .route("/place/:player/:column", post(place))
        .route("/random-board", get(random_board))
        .with_state(RouterState::new())
        .into_service()
}

async fn board(State(state): State<RouterState>) -> Result<Response> {
    let state = state.0.read().await;
    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(state.game_state.to_string().into())?)
}

async fn reset(State(state): State<RouterState>) -> Result<Response> {
    let mut state = state.0.write().await;
    state.game_state.reset();
    state.rng = StdRng::seed_from_u64(2024);
    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(state.game_state.to_string().into())?)
}

async fn place(
    State(state): State<RouterState>,
    Path((player, column)): Path<(Player, usize)>,
) -> Result<Response> {
    let mut state = state.0.write().await;

    if !(1..=BOARD_SIZE).contains(&column) {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::empty())?);
    }

    if state.game_state.column_full(column - 1)
        || state.game_state.board_full()
        || state.game_state.winner().is_some()
    {
        return Ok(Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .body(state.game_state.to_string().into())?);
    }

    state.game_state.play(player, column - 1)?;
    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(state.game_state.to_string().into())?)
}

async fn random_board(State(state): State<RouterState>) -> Result<Response> {
    let mut state = state.0.write().await;
    let random_board = Connect4::random(&mut state.rng);
    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(random_board.to_string().into())?)
}

struct GameState {
    game_state: Connect4,
    rng: StdRng,
}

#[derive(Clone)]
struct RouterState(Arc<RwLock<GameState>>);

impl RouterState {
    fn new() -> Self {
        Self(Arc::new(RwLock::new(GameState {
            game_state: Connect4::new(),
            rng: StdRng::seed_from_u64(2024),
        })))
    }
}
