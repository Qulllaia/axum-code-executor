mod router;
mod controller;
use axum::{Router};

use crate::router::ExecuteRouter;

#[tokio::main]
async fn main() {
    let addr = "127.0.1.1:5000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    println!("Server is running on address {:?}", addr);
    let router = Router::new();
    axum::serve(listener, ExecuteRouter::new(router)).await.unwrap();
}

