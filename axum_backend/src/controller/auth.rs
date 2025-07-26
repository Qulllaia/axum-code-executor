use std::{result, sync::Arc, task::Context};

use axum::response::IntoResponse;
use axum::{body::Body, extract::State, http::StatusCode, Json};
use axum_extra::extract::cookie::{self, SameSite};
use bcrypt::{hash, verify, DEFAULT_COST};
use redis::Connection;
use tokio::sync::Mutex;
use tokio::time;
use crate::email_utils::EmailUtils;
use crate::{auth_utils::AuthUtils, types::AuthBody};
use crate::{types::UserData, Connections};
use axum_extra::{
    TypedHeader,
    headers::authorization::{Authorization, Bearer},
    extract::cookie::{CookieJar, Cookie},
};

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
}