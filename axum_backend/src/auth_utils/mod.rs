use std::env;

use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Utc, Duration};
use jsonwebtoken::{encode,decode, DecodingKey, EncodingKey, Header, Validation};

use crate::types::Claims;

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
            .checked_add_signed(Duration::hours(24))  // Срок жизни 24 часа
            .expect("Invalid timestamp")
            .timestamp() as usize;

        let jwt_token = env::var("JWT_SECRET").unwrap();
        
        let claims = Claims {
            sub: user_id.to_owned(),
            exp: expiration,
        };

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
}