use Rausth::config::{AuthProviderType, SETTINGS};
use serde_json::json;
use vercel_runtime::{Body, Error, Request, Response, StatusCode, run};

#[allow(dead_code)]
#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(_req: Request) -> Result<Response<Body>, Error> {
    // let body = serde_json::to_vec(&json!({ "message": "Hello world" }))?;

    let path = _req.uri().path();

    // normalize and extract the provider segment from paths like "/api/microsoft" or "/api/login/microsoft"
    // return error if provider segment is missing
    let provider_segment = path
        .trim_start_matches('/') // remove leading slash
        .trim_start_matches("api/") // remove leading "api/" if present
        .split('/') // take first segment after "api/"
        .next()
        .ok_or_else(|| Error::from("Provider not specified in the URL"))?;

    // Get AuthProviderType from provider_str
    let auth_provider: AuthProviderType = AuthProviderType::from(provider_segment);
    if auth_provider == AuthProviderType::Unknown(provider_segment.to_string()) {
        return Err(Error::from(format!("Unknown authentication provider: {}", provider_segment)));
    }
    
    let provider_sttings = SETTINGS.auth_providers.iter().find(|p| p.name == auth_provider && p.active).ok_or_else(|| {
        Error::from(format!("Authentication provider '{}' not found or inactive", provider_segment))
    })?;

    let pkce_challenge = "example_challenge"; // Replace with actual PKCE challenge
    let login_url = format!(
        "{}?client_id={}&response_type=code&redirect_uri={}&scope={}&code_challenge={}&code_challenge_method=S256",
        provider_sttings.auth_url,
        provider_sttings.client_id,
        provider_sttings.redirect_uri,
        provider_sttings.scopes,
        pkce_challenge
    );
    let body = serde_json::to_vec(&json!({ "login_url": login_url }))?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(body))?)
}
