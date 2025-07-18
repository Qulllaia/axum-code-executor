mod router;
mod controller;
mod db;
use std::sync::Arc;

use db::connector;
use axum::{Router};
use axum::http::{Method, header, HeaderValue};
use deadpool_postgres::{Manager, Object};
use tower_http::cors::{CorsLayer};
use crate::router::ExecuteRouter;

#[tokio::main]
async fn main() {

     let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);


    let addr = "127.0.1.1:5000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    let database_connection;

    match connector().await {
        Ok(res ) => {
            database_connection = res;
        },
        Err(err) => {
            panic!("ERROR WITH DATABASE CONNECTION: {:?}", err);
        }
    }

    let router: Router<Arc<Object>> = Router::new();
    axum::serve(listener, ExecuteRouter::new(router, database_connection)
        .layer(cors)
        ).await.unwrap();
}

