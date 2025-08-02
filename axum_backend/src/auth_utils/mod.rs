use std::env;

use axum::{http::StatusCode, Json};
use axum_extra::extract::{cookie::{Cookie, SameSite}, CookieJar};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Utc, Duration};
use jsonwebtoken::{encode,decode, DecodingKey, EncodingKey, Header, Validation};
use tokio::sync::MutexGuard;

use crate::{cache::Cache, types::{Claims, UserData}, Connections};

pub struct AuthUtils;

impl AuthUtils {
    pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
        return hash(password, DEFAULT_COST);
    }

    pub fn verify_password(password: &str, hashed_password: &str) -> bool {
       return verify(password, hashed_password).unwrap_or(false)
    }

    pub fn generate_token(user_id: &i64) -> Result<String, jsonwebtoken::errors::Error> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::hours(24)) 
            .expect("Invalid timestamp")
            .timestamp() as usize;

        let jwt_token = env::var("JWT_SECRET").unwrap();
        
        let claims = Claims {
            sub: user_id.to_owned(),
            exp: expiration,
        };

        // println!("{:?}", user_id.to_owned());

        return encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(jwt_token.as_bytes()),
        )
    }

    pub fn validate_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let secret = env::var("JWT_SECRET").unwrap();
        return decode::<Claims>(token, &DecodingKey::from_secret(secret.as_ref()), &Validation::default())
                .map(|data| data.claims);
    }

    pub async fn create_user(mut connection: MutexGuard<'_, Connections>, verify_token_copy: &String, jar: CookieJar) -> Result<(CookieJar,Json<serde_json::Value>), (StatusCode,Json<serde_json::Value>)> {
        if Cache::check_filed_existance(&mut connection, verify_token_copy).await {

            let user_cache_data = Cache::get_data_by_field(&mut connection, &verify_token_copy).await;
            let user_data = serde_json::from_str::<UserData>(&user_cache_data).unwrap();
            let email = user_data.email;
            let password = user_data.password;

            match connection.database.query_one("INSERT INTO \"User\"(email, password) VALUES ($1, $2) RETURNING user_id;", &[&email, &password]).await {
                Ok(result)=>{

                    
                    let user_id: i64 = result.get("user_id");
                    let token = Self::generate_token(&user_id).unwrap();

                    // println!("{:?}", token);

                    let cookie = Cookie::build(("jwt_token", token))
                                                .http_only(true)
                                                .same_site(SameSite::None)
                                                .secure(true)
                                                .path("/")
                                                .build();

                    // println!("{:?}", cookie.value());

                    let jar = jar.add(cookie);

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

        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "result": "no cache data"
            }))
        ))
    }

    pub async fn login_user(connection: MutexGuard<'_, Connections>, password: &str, email: String,  jar: CookieJar) -> Result<(CookieJar,Json<serde_json::Value>), (StatusCode,Json<serde_json::Value>)> {
        // println!("{:?}", &email);
        match connection.database.query_one("SELECT user_id, password FROM \"User\" where email = $1; ", &[&email]).await {
            Ok(result ) => {
                let database_password: String = result.get("password");
                let user_id: i64 = result.get("user_id");
                
                if AuthUtils::verify_password(&password, &database_password) {

                    let token = Self::generate_token(&user_id).unwrap();
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
}