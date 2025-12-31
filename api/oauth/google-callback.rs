use rausth::{auth::{JwtService, GoogleOAuthService}, config::Config, db, models::OAuthCallbackRequest, AuthError};
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};
use serde_json::json;

/// Handle Google OAuth callback.
///
/// This endpoint receives the authorization code from Google after user authentication,
/// exchanges it for an access token, retrieves user information, and issues application tokens.
#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(_req: Request) -> Result<Response<Body>, Error> {
    // Load config
    let config = Config::from_env().map_err(|e| {
        Error::from(format!("Configuration error: {}", e))
    })?;

    // Check if Google OAuth is configured and enabled
    let oauth_config = match &config.auth.google {
        Some(cfg) if cfg.enabled => cfg,
        _ => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from(json!({
                    "error": "Google OAuth is not configured or disabled"
                }).to_string()))?);
        }
    };

    // Parse request body
    let body_bytes = match _req.body() {
        Body::Text(text) => text.as_bytes(),
        Body::Binary(bytes) => bytes.as_slice(),
        Body::Empty => b"",
    };

    let request: OAuthCallbackRequest = serde_json::from_slice(body_bytes)
        .map_err(|e| Error::from(format!("Invalid request: {}", e)))?;

    // Get database service
    let db_service = db::get_db_service().await;

    // Process OAuth callback
    let jwt_service = JwtService::new(config.jwt.clone());
    let oauth_service = GoogleOAuthService::new(jwt_service);

    match oauth_service.handle_callback(oauth_config, request.code, db_service.as_ref()).await {
        Ok(response) => {
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&response)?))?)
        }
        Err(e) => {
            let (status, message) = match e {
                AuthError::OAuthError(msg) => (StatusCode::BAD_REQUEST, msg),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
            };
            Ok(Response::builder()
                .status(status)
                .header("Content-Type", "application/json")
                .body(Body::from(json!({"error": message}).to_string()))?)
        }
    }
}
