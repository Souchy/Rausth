use rausth::{auth::{JwtService, OAuthProvider, OAuthService}, config::Config, AuthError};
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

    let jwt_service = JwtService::new(config.jwt.clone());
    let oauth_service = OAuthService::new(jwt_service);

    match oauth_service.get_authorization_url(OAuthProvider::Microsoft, oauth_config) {
        Ok((auth_url, _)) => {
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
