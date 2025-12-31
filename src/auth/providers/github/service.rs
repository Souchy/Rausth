use super::types::{GitHubEmail, GitHubUserInfo};
use crate::auth::jwt::JwtService;
use crate::config::OAuthProviderConfig;
use crate::models::{AuthResponse, User};
use crate::utils::{AuthError, AuthResult};
use chrono::Utc;
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, RedirectUrl, TokenResponse as OAuth2TokenResponse, TokenUrl,
};
use sqlx::PgPool;
use uuid::Uuid;

/// GitHub OAuth service for handling authentication flows.
///
/// This service manages the OAuth 2.0 flow with GitHub,
/// including generating authorization URLs and handling callbacks.
pub struct GitHubOAuthService {
    jwt_service: JwtService,
}

impl GitHubOAuthService {
    /// Create a new GitHub OAuth service.
    ///
    /// # Arguments
    /// * `jwt_service` - JWT service for generating application tokens
    pub fn new(jwt_service: JwtService) -> Self {
        Self { jwt_service }
    }

    /// Generate an authorization URL for GitHub OAuth.
    ///
    /// This URL should be used to redirect the user to GitHub's login page.
    ///
    /// # Arguments
    /// * `config` - GitHub OAuth provider configuration
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

    /// Handle the OAuth callback from GitHub.
    ///
    /// This method exchanges the authorization code for an access token,
    /// retrieves user information from GitHub API, and creates or finds
    /// the user in our database.
    ///
    /// # Arguments
    /// * `config` - GitHub OAuth provider configuration
    /// * `code` - Authorization code from GitHub
    /// * `db` - Database connection pool
    ///
    /// # Returns
    /// Authentication response with JWT tokens for the application
    pub async fn handle_callback(
        &self,
        config: &OAuthProviderConfig,
        code: String,
        db: &PgPool,
    ) -> AuthResult<AuthResponse> {
        let client = self.create_client(config)?;

        // Exchange the code for an access token
        let token_result = client
            .exchange_code(AuthorizationCode::new(code))
            .request_async(async_http_client)
            .await
            .map_err(|e| AuthError::OAuthError(e.to_string()))?;

        let access_token = token_result.access_token().secret();

        // Get user info from GitHub
        let (provider_id, email) = self.get_user_info(access_token).await?;

        // Find or create user in our database
        let user = self
            .find_or_create_user(db, &email, &provider_id)
            .await?;

        // Generate application tokens
        self.generate_auth_response(db, user.id).await
    }

    /// Create an OAuth client configured for GitHub.
    fn create_client(&self, config: &OAuthProviderConfig) -> AuthResult<BasicClient> {
        let client = BasicClient::new(
            ClientId::new(config.client_id.clone()),
            Some(ClientSecret::new(config.client_secret.clone())),
            AuthUrl::new("https://github.com/login/oauth/authorize".to_string())
                .map_err(|e| AuthError::ConfigError(e.to_string()))?,
            Some(
                TokenUrl::new("https://github.com/login/oauth/access_token".to_string())
                    .map_err(|e| AuthError::ConfigError(e.to_string()))?,
            ),
        )
        .set_redirect_uri(
            RedirectUrl::new(config.redirect_uri.clone())
                .map_err(|e| AuthError::ConfigError(e.to_string()))?,
        );

        Ok(client)
    }

    /// Retrieve user information from GitHub API.
    ///
    /// If the user's email is not publicly available, this method
    /// will fetch it from the /user/emails endpoint.
    async fn get_user_info(&self, access_token: &str) -> AuthResult<(String, String)> {
        let client = reqwest::Client::new();

        // Get user info
        let user_info: GitHubUserInfo = client
            .get("https://api.github.com/user")
            .bearer_auth(access_token)
            .header("User-Agent", "Rausth")
            .send()
            .await
            .map_err(|e| AuthError::OAuthError(e.to_string()))?
            .json()
            .await
            .map_err(|e| AuthError::OAuthError(e.to_string()))?;

        // Get email if not available in user info
        let email = if let Some(email) = user_info.email {
            email
        } else {
            let emails: Vec<GitHubEmail> = client
                .get("https://api.github.com/user/emails")
                .bearer_auth(access_token)
                .header("User-Agent", "Rausth")
                .send()
                .await
                .map_err(|e| AuthError::OAuthError(e.to_string()))?
                .json()
                .await
                .map_err(|e| AuthError::OAuthError(e.to_string()))?;

            emails
                .into_iter()
                .find(|e| e.primary && e.verified)
                .map(|e| e.email)
                .ok_or_else(|| AuthError::OAuthError("No verified email found".to_string()))?
        };

        Ok((user_info.id.to_string(), email))
    }

    /// Find an existing user or create a new one.
    async fn find_or_create_user(
        &self,
        db: &PgPool,
        email: &str,
        provider_id: &str,
    ) -> AuthResult<User> {
        // Try to find existing user
        if let Some(user) = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE provider = $1 AND provider_id = $2",
        )
        .bind("github")
        .bind(provider_id)
        .fetch_optional(db)
        .await?
        {
            return Ok(user);
        }

        // Create new user
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (email, provider, provider_id, created_at, updated_at)
            VALUES ($1, $2, $3, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(email)
        .bind("github")
        .bind(provider_id)
        .fetch_one(db)
        .await?;

        Ok(user)
    }

    /// Generate authentication response with JWT tokens.
    async fn generate_auth_response(
        &self,
        db: &PgPool,
        user_id: Uuid,
    ) -> AuthResult<AuthResponse> {
        let access_token = self.jwt_service.generate_access_token(user_id)?;
        let refresh_token = self.jwt_service.generate_refresh_token();
        let expires_in = self.jwt_service.get_access_token_expiry_seconds();

        // Store refresh token in database
        let refresh_token_expiry = Utc::now()
            + chrono::Duration::seconds(self.jwt_service.get_refresh_token_expiry_seconds());

        sqlx::query(
            r#"
            INSERT INTO refresh_tokens (user_id, token, expires_at, created_at)
            VALUES ($1, $2, $3, NOW())
            "#,
        )
        .bind(user_id)
        .bind(&refresh_token)
        .bind(refresh_token_expiry)
        .execute(db)
        .await?;

        Ok(AuthResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
        })
    }
}
