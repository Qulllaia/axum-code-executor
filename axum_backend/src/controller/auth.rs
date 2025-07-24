use std::{result, sync::Arc, task::Context};

use axum::{body::Body, extract::State, http::StatusCode, Json};
use bcrypt::{hash, verify, DEFAULT_COST};
use redis::Connection;
use tokio::sync::Mutex;
use crate::{auth_utils::AuthUtils, types::AuthBody};

use crate::{types::UserData, Connections};


pub struct AuthController;

impl AuthController {
    pub async fn login_user(
        State(connection): State<Arc<Mutex<Connections>>>,
        Json(user_data): Json<UserData>
    ) -> (StatusCode, Json<serde_json::Value>) {
        let email = user_data.email;
        let password = user_data.password;

        match connection.lock().await.database.query_one("SELECT user_id, password FROM \"User\" where email = $1; ", &[&email]).await {
            Ok(result ) => {
                let database_password: String = result.get("password");
                let user_id: i64 = result.get("user_id");
                
                if AuthUtils::verify_password(&password, &database_password) {
                    return (
                        StatusCode::OK,
                        Json(serde_json::json!(AuthBody {
                            access_token: AuthUtils::generate_token(&user_id).unwrap()
                        }))
                    )
                } else {
                    return (
                        StatusCode::UNAUTHORIZED,
                        Json(serde_json::json!({
                            "result":"done",
                            "error": "Неверный пароль"
                        }))
                    )
                }

            },
            Err(error) => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "result":"done",
                        "error": format!("{:?}", error)
                    }))
                )
            }
        }        
    }

    pub async fn reg_user(
        State(connection): State<Arc<Mutex<Connections>>>,
        Json(user_data): Json<UserData>
    ) -> (StatusCode, Json<serde_json::Value>) {
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

        match connection.lock().await.database.query_one("INSERT INTO \"User\"(email, password) VALUES ($1, $2) RETURNING user_id;", &[&email, &password]).await {
            Ok(result)=>{
                let user_id: i64 = result.get("user_id");
                return (
                    StatusCode::OK,
                    Json(serde_json::json!(AuthBody{
                        access_token: AuthUtils::generate_token(&user_id).unwrap()
                    }))
                )
            },
            Err(error)=>{
                return (
                    StatusCode::OK,
                    Json(serde_json::json!({
                        "result":"error",
                        "error":format!("{:?}", error)
                    }))
                )
            },
        }
    }
}