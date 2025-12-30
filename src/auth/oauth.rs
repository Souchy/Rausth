use crate::config::OAuthProviderConfig;
use crate::models::{AuthResponse, User};
use crate::utils::{AuthError, AuthResult};
use crate::auth::jwt::JwtService;
use chrono::Utc;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl,
    TokenUrl, TokenResponse as OAuth2TokenResponse,
    basic::BasicClient,
    reqwest::async_http_client,
};
use serde::{Deserialize, Serialize};
use sqlx::PgConnection;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct GoogleUserInfo {
    pub id: String,
    pub email: String,
    pub verified_email: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MicrosoftUserInfo {
    pub id: String,
    pub mail: Option<String>,
    pub user_principal_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubUserInfo {
    pub id: u64,
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubEmail {
    pub email: String,
    pub primary: bool,
    pub verified: bool,
}

pub enum OAuthProvider {
    Google,
    Microsoft,
    GitHub,
}

pub struct OAuthService {
    jwt_service: JwtService,
}

impl OAuthService {
    pub fn new(jwt_service: JwtService) -> Self {
        Self { jwt_service }
    }

    pub fn get_authorization_url(
        &self,
        provider: OAuthProvider,
        config: &OAuthProviderConfig,
    ) -> AuthResult<(String, String)> {
        let client = self.create_client(provider, config)?;
        let (auth_url, csrf_token) = client.authorize_url(CsrfToken::new_random).url();
        
        Ok((auth_url.to_string(), csrf_token.secret().clone()))
    }

    pub async fn handle_callback(
        &self,
        provider: OAuthProvider,
        config: &OAuthProviderConfig,
        code: String,
        db: &mut PgConnection,
    ) -> AuthResult<AuthResponse> {
        let client = self.create_client(provider.clone(), config)?;

        // Exchange the code for an access token
        let token_result = client
            .exchange_code(AuthorizationCode::new(code))
            .request_async(async_http_client)
            .await
            .map_err(|e| AuthError::OAuthError(e.to_string()))?;

        let access_token = token_result.access_token().secret();

        // Get user info from the provider
        let (provider_id, email) = match provider {
            OAuthProvider::Google => self.get_google_user_info(access_token).await?,
            OAuthProvider::Microsoft => self.get_microsoft_user_info(access_token).await?,
            OAuthProvider::GitHub => self.get_github_user_info(access_token).await?,
        };

        // Find or create user
        let provider_name = match provider {
            OAuthProvider::Google => "google",
            OAuthProvider::Microsoft => "microsoft",
            OAuthProvider::GitHub => "github",
        };

        let user = self.find_or_create_user(db, &email, &provider_id, provider_name).await?;

        // Generate tokens
        self.generate_auth_response(db, user.id).await
    }

    fn create_client(
        &self,
        provider: OAuthProvider,
        config: &OAuthProviderConfig,
    ) -> AuthResult<BasicClient> {
        let (auth_url, token_url) = match provider {
            OAuthProvider::Google => (
                "https://accounts.google.com/o/oauth2/v2/auth",
                "https://oauth2.googleapis.com/token",
            ),
            OAuthProvider::Microsoft => (
                "https://login.microsoftonline.com/common/oauth2/v2.0/authorize",
                "https://login.microsoftonline.com/common/oauth2/v2.0/token",
            ),
            OAuthProvider::GitHub => (
                "https://github.com/login/oauth/authorize",
                "https://github.com/login/oauth/access_token",
            ),
        };

        let client = BasicClient::new(
            ClientId::new(config.client_id.clone()),
            Some(ClientSecret::new(config.client_secret.clone())),
            AuthUrl::new(auth_url.to_string())
                .map_err(|e| AuthError::ConfigError(e.to_string()))?,
            Some(TokenUrl::new(token_url.to_string())
                .map_err(|e| AuthError::ConfigError(e.to_string()))?),
        )
        .set_redirect_uri(
            RedirectUrl::new(config.redirect_uri.clone())
                .map_err(|e| AuthError::ConfigError(e.to_string()))?,
        );

        Ok(client)
    }

    async fn get_google_user_info(&self, access_token: &str) -> AuthResult<(String, String)> {
        let client = reqwest::Client::new();
        let user_info: GoogleUserInfo = client
            .get("https://www.googleapis.com/oauth2/v2/userinfo")
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| AuthError::OAuthError(e.to_string()))?
            .json()
            .await
            .map_err(|e| AuthError::OAuthError(e.to_string()))?;

        Ok((user_info.id, user_info.email))
    }

    async fn get_microsoft_user_info(&self, access_token: &str) -> AuthResult<(String, String)> {
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

        let email = user_info.mail
            .or(user_info.user_principal_name)
            .ok_or_else(|| AuthError::OAuthError("No email found".to_string()))?;

        Ok((user_info.id, email))
    }

    async fn get_github_user_info(&self, access_token: &str) -> AuthResult<(String, String)> {
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

    async fn find_or_create_user(
        &self,
        db: &mut PgConnection,
        email: &str,
        provider_id: &str,
        provider: &str,
    ) -> AuthResult<User> {
        // Try to find existing user
        if let Some(user) = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE provider = $1 AND provider_id = $2"
        )
        .bind(provider)
        .bind(provider_id)
        .fetch_optional(&mut *db)
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
            "#
        )
        .bind(email)
        .bind(provider)
        .bind(provider_id)
        .fetch_one(&mut *db)
        .await?;

        Ok(user)
    }

    async fn generate_auth_response(
        &self,
        db: &mut PgConnection,
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
            "#
        )
        .bind(user_id)
        .bind(&refresh_token)
        .bind(refresh_token_expiry)
        .execute(&mut *db)
        .await?;

        Ok(AuthResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
        })
    }
}

impl Clone for OAuthProvider {
    fn clone(&self) -> Self {
        match self {
            OAuthProvider::Google => OAuthProvider::Google,
            OAuthProvider::Microsoft => OAuthProvider::Microsoft,
            OAuthProvider::GitHub => OAuthProvider::GitHub,
        }
    }
}
