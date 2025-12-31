use crate::config::JwtConfig;
use crate::models::Claims;
use crate::utils::AuthResult;
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use uuid::Uuid;

/// JWT token service for generating and verifying tokens.
///
/// This service handles creation of access tokens (JWT) and refresh tokens (UUID).
pub struct JwtService {
    config: JwtConfig,
}

impl JwtService {
    /// Create a new JWT service.
    ///
    /// # Arguments
    /// * `config` - JWT configuration including secret and expiry times
    pub fn new(config: JwtConfig) -> Self {
        Self { config }
    }

    /// Generate a JWT access token for a user.
    ///
    /// The access token contains the user ID and expiration time, signed with
    /// the configured secret.
    ///
    /// # Arguments
    /// * `user_id` - The user's UUID
    ///
    /// # Returns
    /// A signed JWT token string
    pub fn generate_access_token(&self, user_id: Uuid) -> AuthResult<String> {
        let now = Utc::now().timestamp() as usize;
        let exp = (Utc::now().timestamp() + self.config.access_token_expiry_seconds) as usize;

        let claims = Claims {
            sub: user_id.to_string(),
            exp,
            iat: now,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.secret.as_bytes()),
        )?;

        Ok(token)
    }

    /// Generate a random refresh token.
    ///
    /// Refresh tokens are UUIDs that are stored in the database.
    ///
    /// # Returns
    /// A UUID string to be used as a refresh token
    pub fn generate_refresh_token(&self) -> String {
        Uuid::new_v4().to_string()
    }

    /// Verify and decode a JWT access token.
    ///
    /// # Arguments
    /// * `token` - The JWT token to verify
    ///
    /// # Returns
    /// The decoded claims if the token is valid
    pub fn verify_token(&self, token: &str) -> AuthResult<Claims> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.config.secret.as_bytes()),
            &Validation::default(),
        )?;

        Ok(token_data.claims)
    }

    /// Get the refresh token expiry time in seconds.
    ///
    /// # Returns
    /// Number of seconds until a refresh token expires
    pub fn get_refresh_token_expiry_seconds(&self) -> i64 {
        self.config.refresh_token_expiry_seconds
    }

    /// Get the access token expiry time in seconds.
    ///
    /// # Returns
    /// Number of seconds until an access token expires
    pub fn get_access_token_expiry_seconds(&self) -> i64 {
        self.config.access_token_expiry_seconds
    }
}
