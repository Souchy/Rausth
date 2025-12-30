use crate::models::{AuthResponse, RefreshToken, RefreshTokenRequest};
use crate::utils::{AuthError, AuthResult};
use crate::auth::jwt::JwtService;
use chrono::Utc;
use sqlx::PgConnection;

pub struct RefreshTokenService {
    jwt_service: JwtService,
}

impl RefreshTokenService {
    pub fn new(jwt_service: JwtService) -> Self {
        Self { jwt_service }
    }

    pub async fn refresh(
        &self,
        db: &mut PgConnection,
        request: RefreshTokenRequest,
    ) -> AuthResult<AuthResponse> {
        // Fetch the refresh token
        let token = sqlx::query_as::<_, RefreshToken>(
            "SELECT * FROM refresh_tokens WHERE token = $1"
        )
        .bind(&request.refresh_token)
        .fetch_optional(&mut *db)
        .await?
        .ok_or(AuthError::InvalidToken)?;

        // Check if token is expired
        if token.expires_at < Utc::now() {
            // Delete expired token
            sqlx::query("DELETE FROM refresh_tokens WHERE id = $1")
                .bind(token.id)
                .execute(&mut *db)
                .await?;
            return Err(AuthError::TokenExpired);
        }

        // Delete old refresh token
        sqlx::query("DELETE FROM refresh_tokens WHERE id = $1")
            .bind(token.id)
            .execute(&mut *db)
            .await?;

        // Generate new tokens
        let access_token = self.jwt_service.generate_access_token(token.user_id)?;
        let new_refresh_token = self.jwt_service.generate_refresh_token();
        let expires_in = self.jwt_service.get_access_token_expiry_seconds();

        // Store new refresh token
        let refresh_token_expiry = Utc::now() 
            + chrono::Duration::seconds(self.jwt_service.get_refresh_token_expiry_seconds());

        sqlx::query(
            r#"
            INSERT INTO refresh_tokens (user_id, token, expires_at, created_at)
            VALUES ($1, $2, $3, NOW())
            "#
        )
        .bind(token.user_id)
        .bind(&new_refresh_token)
        .bind(refresh_token_expiry)
        .execute(&mut *db)
        .await?;

        Ok(AuthResponse {
            access_token,
            refresh_token: new_refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
        })
    }
}
