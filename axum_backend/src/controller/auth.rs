use std::{result, sync::Arc, task::Context};

use axum::extract::Query;
use axum::response::IntoResponse;
use axum::{body::Body, extract::State, http::StatusCode, Json};
use axum_extra::extract::cookie::{self, SameSite};
use bcrypt::{hash, verify, DEFAULT_COST};
use lapin::options::{BasicAckOptions, BasicConsumeOptions, BasicNackOptions, BasicPublishOptions, QueueDeclareOptions, QueueDeleteOptions};
use lapin::types::FieldTable;
use lapin::{BasicProperties, Connection, ConnectionProperties};
use tokio::sync::{oneshot, Mutex};
use tokio::time::{self, sleep, Duration};
use crate::email_utils::EmailUtils;
use crate::types::VerifyToken;
use crate::{auth_utils::AuthUtils, types::AuthBody};
use crate::{types::UserData, Connections};
use axum_extra::{
    TypedHeader,
    headers::authorization::{Authorization, Bearer},
    extract::cookie::{CookieJar, Cookie},
};
use futures_lite::stream::StreamExt;

pub struct AuthController;

impl AuthController {

    // #[axum::debug_handler]
    pub async fn login_user(
        jar: CookieJar,
        State(connection): State<Arc<Mutex<Connections>>>,
        Json(user_data): Json<UserData>
    ) -> Result<(CookieJar, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)>  {
        let email = user_data.email;
        let password = user_data.password;

        match connection.lock().await.database.query_one("SELECT user_id, password FROM \"User\" where email = $1; ", &[&email]).await {
            Ok(result ) => {
                let database_password: String = result.get("password");
                let user_id: i64 = result.get("user_id");
                
                if AuthUtils::verify_password(&password, &database_password) {

                    let token = AuthUtils::generate_token(&user_id).unwrap();
                    let cookie = Cookie::build(("jwt_token", token))
                                                .http_only(true)
                                                .same_site(SameSite::None)
                                                .secure(true)
                                                .path("/")
                                                .build();

                    let jar = jar.add(cookie);
                    return Ok((
                            jar,
                            Json(serde_json::json!({
                                "result":"done",
                                "user_id": user_id,
                            }))
                    ));
                } else {
                    return Err ((
                        StatusCode::UNAUTHORIZED,
                        Json(serde_json::json!({
                            "result":"done",
                            "error": "Неверный пароль"
                        }))
                    ))
                }

            },
            Err(error) => {
                return Err ((
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "result":"done",
                        "error": format!("{:?}", error)
                    }))
                ))
            }
        }        
    }

    // #[axum::debug_handler]
    pub async fn reg_user(
        jar: CookieJar,
        State(connection): State<Arc<Mutex<Connections>>>,
        Json(user_data): Json<UserData>
    ) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)>  {
        let email = user_data.email;
        let password:String;

        match AuthUtils::hash_password(&user_data.password) {
            Ok(result_hash)=>{password = result_hash},
            Err(error) => {
                return Err ((
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(serde_json::json!({
                        "result":"error", 
                        "error": format!("{:?}", error)
                    }))
                ))
            }
        }

        match connection.lock().await.database.query_one("INSERT INTO \"User\"(email, password) VALUES ($1, $2) RETURNING user_id;", &[&email, &password]).await {
            Ok(result)=>{

                
                let user_id: i64 = result.get("user_id");
                let token = AuthUtils::generate_token(&user_id).unwrap();

                println!("{:?}", token);

                let cookie = Cookie::build(("jwt_token", token))
                                            .http_only(true)
                                            .same_site(SameSite::None)
                                            .secure(true)
                                            .path("/")
                                            .build();

                println!("{:?}", cookie.value());

                let jar = jar.add(cookie);
                let _ = EmailUtils::send_verification_email(&"znurock@mail.ru".to_string()).await.unwrap();

                return Ok((
                        jar,
                        Json(serde_json::json!({
                            "result":"done",
                            "user_id": user_id,
                        }))
                ));
            },
            Err(error)=>{
                return Err ((
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "result":"error",
                        "error":format!("{:?}", error)
                    }))
                ))
            },
        }
    }

    pub async fn email_verify(
        State(connection): State<Arc<Mutex<Connections>>>,
        Query(verify_token): Query<VerifyToken>
    ) -> Json<serde_json::Value> {
        println!("{:?}", "email_verify");
            let connection = connection.lock().await;
            let rabbitmq_connection = Connection::connect(
                "amqp://guest:guest@localhost:5672",
                ConnectionProperties::default(),
            )
            .await
            .expect("Failed to connect to RabbitMQ");


            let rabbitmq_channel_producer = rabbitmq_connection.create_channel().await.expect("Failed to create channel");

            let q_name = format!("verification_email");

            rabbitmq_channel_producer.queue_declare(
            q_name.as_str(), 
            QueueDeclareOptions {
                        durable: true,
                        auto_delete: false,
                        ..Default::default()
                    }, 
            FieldTable::default())
            .await
            .expect("Queue crate failed");


            connection.rabbitmq_channel_producer.basic_publish("", 
            q_name.as_str(), 
                BasicPublishOptions::default(), 
                &serde_json::to_vec(&verify_token).unwrap(),
             BasicProperties::default())
                        .await.expect("channel publishing failed");
        return Json(serde_json::json!({
            "result": "matched"
        }))
    }

    pub async fn email_ping_verify(
        State(connection): State<Arc<Mutex<Connections>>>,
        Query(verify_token): Query<VerifyToken>
    ) -> Json<serde_json::Value> {
        let connection = connection.lock().await;
        let _ = connection.rabbitmq_channel_consumer.queue_delete("verification_email", QueueDeleteOptions::default()).await;
        println!("{:?}", "email_ping");

        let q_name = format!("verification_email");
        connection.rabbitmq_channel_consumer.queue_declare(
            q_name.as_str(), 
            QueueDeclareOptions {
                        durable: true,
                        auto_delete: false,
                        ..Default::default()
                    }, 
            FieldTable::default())
            .await
            .expect("Queue crate failed");

        let mut consumer = connection.rabbitmq_channel_consumer.basic_consume(
            q_name.as_str(), 
            "verification_consumer", 
            BasicConsumeOptions::default(), 
            FieldTable::default())
            .await
            .expect("Consumer creation failed");

        
        let timeout_task = tokio::time::timeout(
            std::time::Duration::from_secs(20),
            tokio::spawn(async move {
                let mut result = false;
                
                while let Some(delivery) = consumer.next().await {
                    println!("sfsfsdfs");
                    match delivery {
                        Ok(delivery) => {
                            let data: VerifyToken = serde_json::from_slice(&delivery.data)
                                .expect("Failed to parse message");
                            
                            if data.verify_token == verify_token.verify_token {
                                delivery.ack(BasicAckOptions::default())
                                    .await
                                    .expect("Failed to ack message");
                                result = true;
                                println!("Finded");
                                break;
                            } else {
                                delivery.nack(BasicNackOptions {
                                    multiple: false,
                                    requeue: true
                                })
                                .await
                                .expect("Failed to nack message");
                            }
                        },
                        Err(e) => {
                            eprintln!("Error receiving message: {}", e);
                            break;
                        }
                    }
                }
                
                // let _ = tx.send(result);
            })
        );
        // let _ = timeout_task.await;
        
        return Json(serde_json::json!({
            "result": format!("{:?}", timeout_task.await) 
        }))

    }
}