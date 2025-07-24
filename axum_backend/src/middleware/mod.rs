use axum::{extract::Request, http::StatusCode, middleware::Next, response::IntoResponse, Json};
use serde_json::json;

use crate::auth_utils::AuthUtils;

pub async fn auth_middleware(
    request: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let token = request.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "));

    let token = token.ok_or_else(|| {
        (StatusCode::UNAUTHORIZED, Json(json!({"error": "Missing token"})))
    })?;

    let claims = AuthUtils::validate_token(token).map_err(|_| {
        (StatusCode::UNAUTHORIZED, Json(json!({"error": "Invalid token"})))
    })?;

    let mut request = request;
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}