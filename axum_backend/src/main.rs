mod router;
mod controller;
mod cache;
mod types;
mod db;
mod auth_utils;
mod middleware;
mod email_utils;
mod rabbitmq;

use std::cell::RefCell;
use std::ops::DerefMut;
use std::rc::Rc;
use std::sync::{Arc};

use lapin::{Channel, Connection, ConnectionProperties};
use tokio::sync::Mutex;

use db::connector;
use axum::{Router};
use axum::http::{Method, header, HeaderValue};
use deadpool_postgres::{Manager, Object};
use tower_http::cors::{CorsLayer};
use crate::router::auth::AuthRouter;
use crate::router::executor::ExecuteRouter;
use redis::aio::MultiplexedConnection;
use rabbitmq::email_consumer::EmailConsumer;

pub struct Connections {
    pub database: Object,
    pub redis: MultiplexedConnection,
    pub rabbitmq_channel: Channel
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

    let rabbitmq_connection = Connection::connect(
        "amqp://guest:guest@localhost:5672",
        ConnectionProperties::default(),
    )
    .await
    .expect("Failed to connect to RabbitMQ");
    let rabbitmq_channel = rabbitmq_connection.create_channel().await.expect("Failed to create channel");

    let connections = Arc::new(Mutex::new(Connections{database: database_connection, redis: result_redis_connector, rabbitmq_channel: rabbitmq_channel}));

    axum::serve(listener, 
        Router::new()
        .merge(ExecuteRouter::new(Router::new(), Arc::clone(&connections)))
        .merge(AuthRouter::new(Router::new(), Arc::clone(&connections)))
        .layer(cors)
        ).await.unwrap();
}

