use async_trait::async_trait;
use crate::{
    config::{AuthProviderType, SETTINGS},
    error::{AuthError, AuthResult},
    repo::user_repo::User,
};

#[async_trait]
pub trait AuthService {
    async fn get_login_url(&self, provider: AuthProviderType) -> AuthResult<String>;
    async fn exchange_code_for_token(
        &self,
        provider: AuthProviderType,
        code: &str,
    ) -> AuthResult<String>;
    async fn get_user_info(&self, access_token: &str) -> AuthResult<User>;
    async fn refresh_token(&self, access_token: &str) -> AuthResult<String>;
}

pub struct OAuthService;

#[async_trait]
impl AuthService for OAuthService {
    // implement methods here
    async fn get_login_url(&self, provider: AuthProviderType) -> AuthResult<String> {
        let provider_config = SETTINGS
            .auth_providers
            .iter()
            .find(|p| p.name == provider)
            .ok_or_else(|| AuthError::ConfigError("Provider not found".into()))?;

        let pkce_challenge = "example_challenge"; // Replace with actual PKCE challenge
        Ok(format!(
            "{}?client_id={}&response_type=code&redirect_uri={}&scope={}&code_challenge={}&code_challenge_method=S256",
            provider_config.auth_url,
            provider_config.client_id,
            provider_config.redirect_uri,
            provider_config.scopes,
            pkce_challenge
        ))
    }

    async fn exchange_code_for_token(
        &self,
        provider: AuthProviderType,
        code: &str,
    ) -> AuthResult<String> {
        let provider_config = SETTINGS
            .auth_providers
            .iter()
            .find(|p| p.name == provider)
            .ok_or_else(|| AuthError::ConfigError("Provider not found".into()))?;

        let client = reqwest::Client::new();
        let params = [
            ("client_id", provider_config.client_id.as_str()),
            ("client_secret", provider_config.client_secret.as_str()),
            ("code", code),
            ("redirect_uri", provider_config.redirect_uri.as_str()),
            ("grant_type", "authorization_code"),
        ];
		// Send request to provider's token endpoint
        let resp = client
            .post(&provider_config.token_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| AuthError::NetworkError(format!("Failed to send request: {}", e)))?;

        if !resp.status().is_success() {
            return Err(AuthError::OAuthError(format!(
                "Token exchange failed with status: {}",
                resp.status()
            )));
        }

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| AuthError::OAuthError(format!("Failed to parse response: {}", e)))?;

        let access_token = json
            .get("access_token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AuthError::OAuthError("Access token not found in response".into()))?;

        Ok(access_token.to_string())
    }

    async fn get_user_info(&self, access_token: &str) -> AuthResult<User> {
        // Implement user info retrieval logic here
        Err(AuthError::NotImplemented(
            "get_user_info not implemented".into(),
        ))
    }

    async fn refresh_token(&self, access_token: &str) -> AuthResult<String> {
        // Implement token refresh logic here
        Err(AuthError::NotImplemented(
            "refresh_token not implemented".into(),
        ))
    }
}
