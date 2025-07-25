use axum::{extract::Request, http::StatusCode, middleware::Next, response::IntoResponse, Json};
use axum_extra::{extract::CookieJar, headers::{authorization::Bearer, Authorization}, TypedHeader};
use serde_json::json;

use crate::auth_utils::AuthUtils;

pub async fn auth_middleware(
    jar: CookieJar,
    request: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    println!("{:?}", jar);

    let token = jar.get("jwt_token").ok_or_else(|| {
        (StatusCode::UNAUTHORIZED, Json(json!({"error": "Missing token"})))
    }).to_owned()?;

    println!("{:?}", token);
    
    let claims = AuthUtils::validate_token(token.value()).map_err(|_| {
        (StatusCode::UNAUTHORIZED, Json(json!({"error": "Invalid token"})))
    })?;

    let mut request = request;
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}