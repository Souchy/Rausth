use rausth::{auth::{JwtService, OAuthProvider, OAuthService}, config::Config, db, models::OAuthCallbackRequest, AuthError};
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(_req: Request) -> Result<Response<Body>, Error> {
    let config = Config::from_env().map_err(|e| Error::from(format!("Configuration error: {}", e)))?;
    let oauth_config = match &config.auth.microsoft {
        Some(cfg) if cfg.enabled => cfg,
        _ => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from(json!({"error": "Microsoft OAuth is not configured or disabled"}).to_string()))?);
        }
    };

    let body_bytes = match _req.body() {
        Body::Text(text) => text.as_bytes(),
        Body::Binary(bytes) => bytes.as_slice(),
        Body::Empty => b"",
    };

    let request: OAuthCallbackRequest = serde_json::from_slice(body_bytes)
        .map_err(|e| Error::from(format!("Invalid request: {}", e)))?;

    let pool = db::get_pool().await.map_err(|e| Error::from(format!("Database error: {}", e)))?;
    let jwt_service = JwtService::new(config.jwt.clone());
    let oauth_service = OAuthService::new(jwt_service);

    match oauth_service.handle_callback(OAuthProvider::Microsoft, oauth_config, request.code, pool).await {
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
