mod router;
mod controller;
use axum::{Router};
use axum::http::{Method, header, HeaderValue};
use tower_http::cors::{Any, CorsLayer};
use crate::router::ExecuteRouter;

#[tokio::main]
async fn main() {

     let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);


    let addr = "127.0.1.1:5000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    println!("Server is running on address {:?}", addr);
    let router = Router::new();
    axum::serve(listener, ExecuteRouter::new(router)
        .layer(cors)).await.unwrap();
}

