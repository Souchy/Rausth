use super::types::MicrosoftUserInfo;
use crate::auth::jwt::JwtService;
use crate::config::OAuthProviderConfig;
use crate::db::DatabaseService;
use crate::models::{AuthResponse, User};
use crate::utils::{AuthError, AuthResult};
use chrono::Utc;
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, RedirectUrl, TokenResponse as OAuth2TokenResponse, TokenUrl,
};
use uuid::Uuid;

/// Microsoft OAuth service for handling authentication flows.
///
/// This service manages the OAuth 2.0 flow with Microsoft (Azure AD),
/// including generating authorization URLs and handling callbacks.
pub struct MicrosoftOAuthService {
    jwt_service: JwtService,
}

impl MicrosoftOAuthService {
    /// Create a new Microsoft OAuth service.
    ///
    /// # Arguments
    /// * `jwt_service` - JWT service for generating application tokens
    pub fn new(jwt_service: JwtService) -> Self {
        Self { jwt_service }
    }

    /// Generate an authorization URL for Microsoft OAuth.
    ///
    /// This URL should be used to redirect the user to Microsoft's login page.
    ///
    /// # Arguments
    /// * `config` - Microsoft OAuth provider configuration
    ///
    /// # Returns
    /// A tuple containing the authorization URL and CSRF token
    pub fn get_authorization_url(
        &self,
        config: &OAuthProviderConfig,
    ) -> AuthResult<(String, String)> {
        let client = self.create_client(config)?;
        let (auth_url, csrf_token) = client.authorize_url(CsrfToken::new_random).url();

        Ok((auth_url.to_string(), csrf_token.secret().clone()))
    }

    /// Handle the OAuth callback from Microsoft.
    ///
    /// This method exchanges the authorization code for an access token,
    /// retrieves user information from Microsoft Graph API, and creates
    /// or finds the user in our database.
    ///
    /// # Arguments
    /// * `config` - Microsoft OAuth provider configuration
    /// * `code` - Authorization code from Microsoft
    /// * `db` - Database service for user management
    ///
    /// # Returns
    /// Authentication response with JWT tokens for the application
    pub async fn handle_callback(
        &self,
        config: &OAuthProviderConfig,
        code: String,
        db: &dyn DatabaseService,
    ) -> AuthResult<AuthResponse> {
        let client = self.create_client(config)?;

        // Exchange the code for an access token
        let token_result = client
            .exchange_code(AuthorizationCode::new(code))
            .request_async(async_http_client)
            .await
            .map_err(|e| AuthError::OAuthError(e.to_string()))?;

        let access_token = token_result.access_token().secret();

        // Get user info from Microsoft
        let (provider_id, email) = self.get_user_info(access_token).await?;

        // Find or create user in our database
        let user = self
            .find_or_create_user(db, &email, &provider_id)
            .await?;

        // Generate application tokens
        self.generate_auth_response(db, user.id).await
    }

    /// Create an OAuth client configured for Microsoft.
    fn create_client(&self, config: &OAuthProviderConfig) -> AuthResult<BasicClient> {
        let client = BasicClient::new(
            ClientId::new(config.client_id.clone()),
            Some(ClientSecret::new(config.client_secret.clone())),
            AuthUrl::new(
                "https://login.microsoftonline.com/common/oauth2/v2.0/authorize".to_string(),
            )
            .map_err(|e| AuthError::ConfigError(e.to_string()))?,
            Some(
                TokenUrl::new(
                    "https://login.microsoftonline.com/common/oauth2/v2.0/token".to_string(),
                )
                .map_err(|e| AuthError::ConfigError(e.to_string()))?,
            ),
        )
        .set_redirect_uri(
            RedirectUrl::new(config.redirect_uri.clone())
                .map_err(|e| AuthError::ConfigError(e.to_string()))?,
        );

        Ok(client)
    }

    /// Retrieve user information from Microsoft Graph API.
    async fn get_user_info(&self, access_token: &str) -> AuthResult<(String, String)> {
        let client = reqwest::Client::new();
        let user_info: MicrosoftUserInfo = client
            .get("https://graph.microsoft.com/v1.0/me")
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| AuthError::OAuthError(e.to_string()))?
            .json()
            .await
            .map_err(|e| AuthError::OAuthError(e.to_string()))?;

        let email = user_info
            .mail
            .or(user_info.user_principal_name)
            .ok_or_else(|| AuthError::OAuthError("No email found".to_string()))?;

        Ok((user_info.id, email))
    }

    /// Find an existing user or create a new one using the database service.
    async fn find_or_create_user(
        &self,
        db: &dyn DatabaseService,
        email: &str,
        provider_id: &str,
    ) -> AuthResult<User> {
        // Try to find existing user
        if let Some(user) = db.find_user_by_provider("microsoft", provider_id).await? {
            return Ok(user);
        }

        // Create new user
        let user = db.create_user(email, None, "microsoft", Some(provider_id)).await?;
        Ok(user)
    }

    /// Generate authentication response with JWT tokens.
    async fn generate_auth_response(
        &self,
        db: &dyn DatabaseService,
        user_id: Uuid,
    ) -> AuthResult<AuthResponse> {
        let access_token = self.jwt_service.generate_access_token(user_id)?;
        let refresh_token = self.jwt_service.generate_refresh_token();
        let expires_in = self.jwt_service.get_access_token_expiry_seconds();

        // Store refresh token in database
        let refresh_token_expiry = Utc::now()
            + chrono::Duration::seconds(self.jwt_service.get_refresh_token_expiry_seconds());

        db.store_refresh_token(user_id, &refresh_token, refresh_token_expiry).await?;

        Ok(AuthResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
        })
    }
}
