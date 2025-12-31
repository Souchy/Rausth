use rausth::{auth::{JwtService, RefreshTokenService}, config::Config, db, models::RefreshTokenRequest, AuthError};
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};
use serde_json::json;

/// Handle token refresh request.
///
/// This endpoint exchanges a valid refresh token for a new access token and refresh token.
/// The old refresh token is invalidated upon successful refresh.
///
/// # Request Body
/// ```json
/// {
///   "refresh_token": "uuid-refresh-token"
/// }
/// ```
///
/// # Response
/// Returns new JWT access token and refresh token on success.
#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(_req: Request) -> Result<Response<Body>, Error> {
    // Load config
    let config = Config::from_env().map_err(|e| {
        Error::from(format!("Configuration error: {}", e))
    })?;

    // Parse request body
    let body_bytes = match _req.body() {
        Body::Text(text) => text.as_bytes(),
        Body::Binary(bytes) => bytes.as_slice(),
        Body::Empty => b"",
    };

    let request: RefreshTokenRequest = serde_json::from_slice(body_bytes)
        .map_err(|e| Error::from(format!("Invalid request: {}", e)))?;

    // Get database pool
    let pool = db::get_pool().await
        .map_err(|e| Error::from(format!("Database error: {}", e)))?;

    // Process token refresh
    let jwt_service = JwtService::new(config.jwt.clone());
    let refresh_service = RefreshTokenService::new(jwt_service);
    
    match refresh_service.refresh(pool, request).await {
        Ok(response) => {
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&response)?))?)
        }
        Err(e) => {
            let (status, message) = match e {
                AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token"),
                AuthError::TokenExpired => (StatusCode::UNAUTHORIZED, "Token expired"),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
            };
            Ok(Response::builder()
                .status(status)
                .header("Content-Type", "application/json")
                .body(Body::from(json!({"error": message}).to_string()))?)
        }
    }
}
