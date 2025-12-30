use crate::config::JwtConfig;
use crate::models::Claims;
use crate::utils::AuthResult;
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use uuid::Uuid;

pub struct JwtService {
    config: JwtConfig,
}

impl JwtService {
    pub fn new(config: JwtConfig) -> Self {
        Self { config }
    }

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

    pub fn generate_refresh_token(&self) -> String {
        Uuid::new_v4().to_string()
    }

    pub fn verify_token(&self, token: &str) -> AuthResult<Claims> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.config.secret.as_bytes()),
            &Validation::default(),
        )?;

        Ok(token_data.claims)
    }

    pub fn get_refresh_token_expiry_seconds(&self) -> i64 {
        self.config.refresh_token_expiry_seconds
    }

    pub fn get_access_token_expiry_seconds(&self) -> i64 {
        self.config.access_token_expiry_seconds
    }
}
