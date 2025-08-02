use std::sync::Arc;

use axum::extract::Query;
use axum::{extract::State, http::StatusCode, Json};
use lapin::options::QueueDeleteOptions;
use tokio::sync::Mutex;
use crate::cache::Cache;
use crate::email_utils::EmailUtils;
use crate::rabbitmq::EmailConsumer;
use crate::types::VerifyToken;
use crate::auth_utils::AuthUtils;
use crate::{types::UserData, Connections};
use axum_extra::
    extract::cookie::CookieJar
;

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
        let connection = connection.lock().await;

        match AuthUtils::login_user(connection, password.as_str(), email, jar).await {
            Ok(result) => {
                return Ok(result)
            },
            Err(error) => {
                return Err(error)
            }
        }
               
    }

    // #[axum::debug_handler]
    pub async fn reg_user(
        State(connection): State<Arc<Mutex<Connections>>>,
        Json(user_data): Json<UserData>
    ) -> (StatusCode, Json<serde_json::Value>)  {
        let email = user_data.email;
        let password:String;

        match AuthUtils::hash_password(&user_data.password) {
            Ok(result_hash)=>{password = result_hash},
            Err(error) => {
                return (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(serde_json::json!({
                        "result":"error", 
                        "error": format!("{:?}", error)
                    }))
                )
            }
        }

        let verify_token = uuid::Uuid::new_v4();

        let email_clone = email.clone();

        let user_data = UserData{
            email: email,
            password: password,
        };

        let _ = Cache::set_data_by_field(&mut connection.lock().await, &verify_token.to_string(), &serde_json::to_string(&user_data).unwrap()).await;

        let _ = EmailUtils::send_verification_email(&email_clone.to_string(), &verify_token.to_string()).await.unwrap();

        return (
            StatusCode::OK,
            Json(serde_json::json!({
                "verify_result": verify_token.to_string()
            }))
        )

    }

    pub async fn email_ping_verify(
        jar: CookieJar,
        State(connection): State<Arc<Mutex<Connections>>>,
        Query(verify_token): Query<VerifyToken>
    ) -> Result<(CookieJar,Json<serde_json::Value>), (StatusCode,Json<serde_json::Value>)>  {
        let connection = connection.lock().await;
        let _ = connection.rabbitmq_channel_consumer.queue_delete("verification_email", QueueDeleteOptions::default()).await;

        let verify_token_copy = verify_token.verify_token.clone();

        let q_name = format!("verification_email");

        let verify_token_string = verify_token.verify_token;

        let timeout_task = EmailConsumer::get_verification_ping(&connection, &q_name, verify_token_string).await;

        let result_query = timeout_task.await.unwrap().unwrap();

        if result_query {
            match AuthUtils::create_user(connection, &verify_token_copy, jar).await {
                Ok(result) => {
                    return Ok(result);
                }
                Err(error) => {
                    return Err(error);
                }
            };
        }

        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "verify_result": format!("{:?}", result_query), 
            }))
        )) 
    }
}


// RIP моя идея сделать консумера и продусера просто в разных эндпоинтах 
// Просто сохраняю самому себе на память, вдруг пригодится 

// pub async fn email_verify(
//     State(connection): State<Arc<Mutex<Connections>>>,
//     Query(verify_token): Query<VerifyToken>
// ) -> Json<serde_json::Value> {
//         let connection = connection.lock().await;
//         let rabbitmq_connection = Connection::connect(
//             "amqp://guest:guest@localhost:5672",
//             ConnectionProperties::default(),
//         )
//         .await
//         .expect("Failed to connect to RabbitMQ");


//         let rabbitmq_channel_producer = rabbitmq_connection.create_channel().await.expect("Failed to create channel");

//         let q_name = format!("verification_email");

//         rabbitmq_channel_producer.queue_declare(
//         q_name.as_str(), 
//         QueueDeclareOptions {
//                     durable: true,
//                     auto_delete: false,
//                     ..Default::default()
//                 }, 
//         FieldTable::default())
//         .await
//         .expect("Queue crate failed");


//         connection.rabbitmq_channel_producer.basic_publish("", 
//         q_name.as_str(), 
//             BasicPublishOptions::default(), 
//             &serde_json::to_vec(&verify_token).unwrap(),
//          BasicProperties::default())
//                     .await.expect("channel publishing failed");
//     return Json(serde_json::json!({
//         "result": "matched"
//     }))
// }