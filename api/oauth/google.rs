use rausth::{auth::{JwtService, GoogleOAuthService}, config::Config};
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};
use serde_json::json;

/// Handle Google OAuth authorization request.
///
/// Generates an authorization URL and redirects the user to Google's login page.
/// The user will be redirected back to the callback URL after authentication.
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

    // Generate authorization URL
    let jwt_service = JwtService::new(config.jwt.clone());
    let oauth_service = GoogleOAuthService::new(jwt_service);

    match oauth_service.get_authorization_url(oauth_config) {
        Ok((auth_url, _csrf_token)) => {
            Ok(Response::builder()
                .status(StatusCode::TEMPORARY_REDIRECT)
                .header("Location", auth_url)
                .body(Body::Empty)?)
        }
        Err(e) => {
            Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header("Content-Type", "application/json")
                .body(Body::from(json!({"error": format!("{}", e)}).to_string()))?)
        }
    }
}
