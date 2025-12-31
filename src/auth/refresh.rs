use crate::models::{AuthResponse, RefreshTokenRequest};
use crate::utils::{AuthError, AuthResult};
use crate::auth::jwt::JwtService;
use crate::db::DatabaseService;
use chrono::Utc;

/// Refresh token service for rotating application tokens.
///
/// This service handles the exchange of refresh tokens for new access tokens,
/// implementing token rotation for enhanced security.
pub struct RefreshTokenService {
    jwt_service: JwtService,
}

impl RefreshTokenService {
    /// Create a new refresh token service.
    ///
    /// # Arguments
    /// * `jwt_service` - JWT service for generating new tokens
    pub fn new(jwt_service: JwtService) -> Self {
        Self { jwt_service }
    }

    /// Exchange a refresh token for new access and refresh tokens.
    ///
    /// This method validates the refresh token, checks expiration, and generates
    /// new tokens. The old refresh token is deleted from the database (token rotation).
    ///
    /// # Arguments
    /// * `db` - Database service for token management
    /// * `request` - Refresh token request containing the refresh token
    ///
    /// # Returns
    /// New authentication response with fresh tokens
    pub async fn refresh(
        &self,
        db: &dyn DatabaseService,
        request: RefreshTokenRequest,
    ) -> AuthResult<AuthResponse> {
        // Fetch the refresh token
        let token = db.find_refresh_token(&request.refresh_token)
            .await?
            .ok_or(AuthError::InvalidToken)?;

        // Check if token is expired
        if token.expires_at < Utc::now() {
            // Delete expired token
            db.delete_refresh_token(token.id).await?;
            return Err(AuthError::TokenExpired);
        }

        // Delete old refresh token (token rotation)
        db.delete_refresh_token(token.id).await?;

        // Generate new tokens
        let access_token = self.jwt_service.generate_access_token(token.user_id)?;
        let new_refresh_token = self.jwt_service.generate_refresh_token();
        let expires_in = self.jwt_service.get_access_token_expiry_seconds();

        // Store new refresh token
        let refresh_token_expiry = Utc::now() 
            + chrono::Duration::seconds(self.jwt_service.get_refresh_token_expiry_seconds());

        db.store_refresh_token(token.user_id, &new_refresh_token, refresh_token_expiry).await?;

        Ok(AuthResponse {
            access_token,
            refresh_token: new_refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
        })
    }
}
