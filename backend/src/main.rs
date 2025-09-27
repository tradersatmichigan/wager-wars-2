use backend::{api::{join_game, place_bet, AppState}, game::Game};
use tokio::net::TcpListener;
use axum::{routing::post, Router};

#[tokio::main]
async fn main() {

    let game = Game::new(vec![]);
    let state = AppState::new(game);

    let app = Router::new()
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    let _ = axum::serve(listener, app);

    loop {
        println!("Type enter to progress to the next stage");
    }
}
