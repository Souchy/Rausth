use crate::models::{User, RefreshToken};
use crate::utils::AuthResult;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Database service trait for abstracting database operations.
/// 
/// This trait allows the authentication system to work with different
/// database implementations (PostgreSQL, MySQL, SQLite, etc.) by
/// providing a common interface for all database operations.
#[async_trait]
pub trait DatabaseService: Send + Sync {
    /// Initialize the database connection and run migrations.
    async fn initialize(&self) -> AuthResult<()>;
    
    /// Find a user by email and authentication provider.
    ///
    /// # Arguments
    /// * `email` - The user's email address
    /// * `provider` - The authentication provider name (e.g., "email", "google")
    async fn find_user_by_email_and_provider(
        &self,
        email: &str,
        provider: &str,
    ) -> AuthResult<Option<User>>;
    
    /// Find a user by provider and provider ID.
    ///
    /// # Arguments
    /// * `provider` - The authentication provider name
    /// * `provider_id` - The user's ID from the provider
    async fn find_user_by_provider(
        &self,
        provider: &str,
        provider_id: &str,
    ) -> AuthResult<Option<User>>;
    
    /// Create a new user in the database.
    ///
    /// # Arguments
    /// * `email` - The user's email address
    /// * `password_hash` - Optional hashed password for email/password auth
    /// * `provider` - The authentication provider name
    /// * `provider_id` - Optional provider ID for OAuth providers
    async fn create_user(
        &self,
        email: &str,
        password_hash: Option<&str>,
        provider: &str,
        provider_id: Option<&str>,
    ) -> AuthResult<User>;
    
    /// Store a refresh token in the database.
    ///
    /// # Arguments
    /// * `user_id` - The user's UUID
    /// * `token` - The refresh token string
    /// * `expires_at` - When the token expires
    async fn store_refresh_token(
        &self,
        user_id: Uuid,
        token: &str,
        expires_at: DateTime<Utc>,
    ) -> AuthResult<()>;
    
    /// Find a refresh token by its value.
    ///
    /// # Arguments
    /// * `token` - The refresh token string to search for
    async fn find_refresh_token(&self, token: &str) -> AuthResult<Option<RefreshToken>>;
    
    /// Delete a refresh token from the database.
    ///
    /// # Arguments
    /// * `token_id` - The UUID of the refresh token to delete
    async fn delete_refresh_token(&self, token_id: Uuid) -> AuthResult<()>;
}
