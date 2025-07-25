mod router;
mod controller;
mod cache;
mod types;
mod db;
mod auth_utils;
mod middleware;
mod email_utils;

use std::cell::RefCell;
use std::ops::DerefMut;
use std::rc::Rc;
use std::sync::{Arc};

use tokio::sync::Mutex;

use db::connector;
use axum::{Router};
use axum::http::{Method, header, HeaderValue};
use deadpool_postgres::{Manager, Object};
use tower_http::cors::{CorsLayer};
use crate::router::auth::AuthRouter;
use crate::router::executor::ExecuteRouter;
use redis::aio::MultiplexedConnection;
pub struct Connections {
    pub database: Object,
    pub redis: MultiplexedConnection 
}

#[tokio::main]
async fn main() {

     let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_credentials(true)
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION, header::COOKIE])
        .expose_headers([header::SET_COOKIE]);


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

    let redis_connection;
    let redis_client = redis::Client::open("redis://127.0.0.1/");
    match redis_client {
        Ok(client)=>{
            redis_connection = client.get_multiplexed_async_connection().await;
        },
        Err(redis_error) => {panic!("Redis Open Error {:?}", redis_error)},
    }

    let mut result_redis_connector;
    match redis_connection {
        Ok(connector )=>{ result_redis_connector = connector},
        Err(redis_error) => {panic!("Redis Connection Error {:?}", redis_error)},
    }

    let connections = Arc::new(Mutex::new(Connections{database: database_connection, redis: result_redis_connector}));

    axum::serve(listener, 
        Router::new()
        .merge(ExecuteRouter::new(Router::new(), Arc::clone(&connections)))
        .merge(AuthRouter::new(Router::new(), Arc::clone(&connections)))
        .layer(cors)
        ).await.unwrap();
}

