use rausth::{auth::{EmailPasswordAuth, JwtService}, config::Config, db, models::LoginRequest, AuthError};
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};
use serde_json::json;

/// Handle user login with email and password.
///
/// This endpoint authenticates a user with their email and password.
/// On successful authentication, returns JWT tokens for accessing the application.
///
/// # Request Body
/// ```json
/// {
///   "email": "user@example.com",
///   "password": "securepassword123"
/// }
/// ```
///
/// # Response
/// Returns JWT access token and refresh token on success.
#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(_req: Request) -> Result<Response<Body>, Error> {
    // Load config
    let config = Config::from_env().map_err(|e| {
        Error::from(format!("Configuration error: {}", e))
    })?;

    // Check if email/password auth is enabled
    if !config.auth.email_password {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(json!({
                "error": "Email/password authentication is disabled"
            }).to_string()))?);
    }

    // Parse request body
    let body_bytes = match _req.body() {
        Body::Text(text) => text.as_bytes(),
        Body::Binary(bytes) => bytes.as_slice(),
        Body::Empty => b"",
    };

    let request: LoginRequest = serde_json::from_slice(body_bytes)
        .map_err(|e| Error::from(format!("Invalid request: {}", e)))?;

    // Get database pool
    let pool = db::get_pool().await
        .map_err(|e| Error::from(format!("Database error: {}", e)))?;

    // Process login
    let jwt_service = JwtService::new(config.jwt.clone());
    let auth = EmailPasswordAuth::new(jwt_service);
    
    match auth.login(pool, request).await {
        Ok(response) => {
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&response)?))?)
        }
        Err(e) => {
            let (status, message) = match e {
                AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials"),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
            };
            Ok(Response::builder()
                .status(status)
                .header("Content-Type", "application/json")
                .body(Body::from(json!({"error": message}).to_string()))?)
        }
    }
}
