use std::{result, sync::Arc, task::Context};

use axum::extract::Query;
use axum::response::IntoResponse;
use axum::{body::Body, extract::State, http::StatusCode, Json};
use axum_extra::extract::cookie::{self, SameSite};
use bcrypt::{hash, verify, DEFAULT_COST};
use lapin::options::{BasicAckOptions, BasicConsumeOptions, BasicNackOptions, BasicPublishOptions, QueueDeclareOptions};
use lapin::types::FieldTable;
use lapin::BasicProperties;
use redis::Connection;
use tokio::sync::Mutex;
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
        tokio::spawn(async move {
            let connection = connection.lock().await;

            let q_name = format!("verification_email {}", verify_token.verify_token);

            connection.rabbitmq_channel.basic_publish("", 
                q_name.as_str(), 
                BasicPublishOptions::default(), 
                &serde_json::to_vec(&verify_token).unwrap(),
             BasicProperties::default())
                        .await.expect("channel publishing failed");
        });
        return Json(serde_json::json!({
            "result": "matched"
        }))
    }

    pub async fn email_ping_verify(
        State(connection): State<Arc<Mutex<Connections>>>,
        Query(verify_token): Query<VerifyToken>
    ) -> Json<serde_json::Value> {
        let connection = connection.lock().await;
        println!("{:?}", "email_ping");

        let q_name = format!("verification_email {}", verify_token.verify_token);
        connection.rabbitmq_channel.queue_declare(
            q_name.as_str(), 
            QueueDeclareOptions::default(), 
            FieldTable::default())
            .await
            .expect("Queue crate failed");

        let mut consumer = connection.rabbitmq_channel.basic_consume(
            q_name.as_str(), 
            "verification_consumer", 
            BasicConsumeOptions::default(), 
            FieldTable::default())
            .await
            .expect("Consumer creation failed");

        let delivery = tokio::time::timeout(
            std::time::Duration::from_secs(20), 
            tokio::spawn( async move {
                while let Some(delivery) = consumer.next().await {
                    match delivery {
                        Ok(delivery) => {
                            println!("{:?}", delivery);
                            let data:VerifyToken = serde_json::from_slice(&delivery.data).expect("Failed to convert into verify");
                            if data.verify_token == verify_token.verify_token {
                                delivery.ack(BasicAckOptions::default())
                                    .await
                                .expect("Failed to ack message");
                            return Ok(true);
                        } else {
                            delivery.nack(BasicNackOptions { 
                                        multiple: false, 
                                        requeue: true })
                                        .await
                                        .expect("Failed to ack message");
                                    continue;
                                }
                                
                            },
                            Err(e) => {
                                eprintln!("Error receiving message: {}", e);
                                return Err(false);
                            }
                    }
                }
                Err(false)
             })
        ).await;
        return Json(serde_json::json!({
            "result": format!("{:?}", delivery) 
        }))

    }
}