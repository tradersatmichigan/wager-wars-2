use tokio::net::TcpListener;
use axum::Router;

#[tokio::main]
async fn main() {
    let app = Router::new();

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
