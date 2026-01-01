use Rausth::config::{AuthProviderType, SETTINGS};
use vercel_runtime::{Body, Error, Request, Response, StatusCode, run};

#[allow(dead_code)]
#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

/**
 * /api/signin/{provider}
 * e.g. /api/signin/microsoft
 * Handles the sign-in request by redirecting to the appropriate authentication provider's login URL.
 */
pub async fn handler(_req: Request) -> Result<Response<Body>, Error> {
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
    let provider_type: AuthProviderType = AuthProviderType::from(provider_segment);
    if provider_type == AuthProviderType::Unknown(provider_segment.to_string()) {
        return Err(Error::from(format!("Unknown authentication provider: {}", provider_segment)));
    }
    
    let provider_settings = SETTINGS.auth_providers.iter()
        .find(|p| p.name == provider_type && p.enabled).ok_or_else(|| {
            Error::from(format!("Authentication provider '{}' not found or inactive", provider_segment))
        })?;

    let pkce_challenge = "example_challenge"; // Replace with actual PKCE challenge
    let login_url = format!(
        "{}?client_id={}&response_type=code&redirect_uri={}&scope={}&code_challenge={}&code_challenge_method=S256",
        provider_settings.auth_url,
        provider_settings.client_id,
        provider_settings.redirect_uri,
        provider_settings.scopes,
        pkce_challenge
    );

    // return as a redirect response
    Ok(Response::builder()
        .status(StatusCode::FOUND)
        .header("Location", login_url)
        .body(Body::Empty)?
    )
}
