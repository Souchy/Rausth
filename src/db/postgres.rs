use super::service::DatabaseService;
use crate::models::{RefreshToken, User};
use crate::utils::AuthResult;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use once_cell::sync::OnceCell;
use sqlx::{postgres::PgPoolOptions, PgPool};
use uuid::Uuid;

static DB_POOL: OnceCell<PgPool> = OnceCell::new();

/// PostgreSQL implementation of the DatabaseService trait.
///
/// This service manages a connection pool to a PostgreSQL database
/// and provides all database operations needed for authentication.
pub struct PostgresService;

impl PostgresService {
    /// Create a new PostgreSQL service instance.
    pub fn new() -> Self {
        Self
    }

    /// Get the database connection pool, initializing it if necessary.
    ///
    /// The pool is created once and reused for all subsequent requests.
    async fn get_pool(&self) -> Result<&'static PgPool, sqlx::Error> {
        if let Some(pool) = DB_POOL.get() {
            return Ok(pool);
        }

        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await?;
        
        DB_POOL.set(pool).map_err(|_| sqlx::Error::PoolClosed)?;
        Ok(DB_POOL.get().unwrap())
    }
}

#[async_trait]
impl DatabaseService for PostgresService {
    async fn initialize(&self) -> AuthResult<()> {
        let pool = self.get_pool().await?;
        
        // Run migrations
        sqlx::query(INIT_SQL)
            .execute(pool)
            .await?;
        
        Ok(())
    }

    async fn find_user_by_email_and_provider(
        &self,
        email: &str,
        provider: &str,
    ) -> AuthResult<Option<User>> {
        let pool = self.get_pool().await?;
        
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE email = $1 AND provider = $2"
        )
        .bind(email)
        .bind(provider)
        .fetch_optional(pool)
        .await?;
        
        Ok(user)
    }

    async fn find_user_by_provider(
        &self,
        provider: &str,
        provider_id: &str,
    ) -> AuthResult<Option<User>> {
        let pool = self.get_pool().await?;
        
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE provider = $1 AND provider_id = $2"
        )
        .bind(provider)
        .bind(provider_id)
        .fetch_optional(pool)
        .await?;
        
        Ok(user)
    }

    async fn create_user(
        &self,
        email: &str,
        password_hash: Option<&str>,
        provider: &str,
        provider_id: Option<&str>,
    ) -> AuthResult<User> {
        let pool = self.get_pool().await?;
        
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (email, password_hash, provider, provider_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(email)
        .bind(password_hash)
        .bind(provider)
        .bind(provider_id)
        .fetch_one(pool)
        .await?;
        
        Ok(user)
    }

    async fn store_refresh_token(
        &self,
        user_id: Uuid,
        token: &str,
        expires_at: DateTime<Utc>,
    ) -> AuthResult<()> {
        let pool = self.get_pool().await?;
        
        sqlx::query(
            r#"
            INSERT INTO refresh_tokens (user_id, token, expires_at, created_at)
            VALUES ($1, $2, $3, NOW())
            "#
        )
        .bind(user_id)
        .bind(token)
        .bind(expires_at)
        .execute(pool)
        .await?;
        
        Ok(())
    }

    async fn find_refresh_token(&self, token: &str) -> AuthResult<Option<RefreshToken>> {
        let pool = self.get_pool().await?;
        
        let refresh_token = sqlx::query_as::<_, RefreshToken>(
            "SELECT * FROM refresh_tokens WHERE token = $1"
        )
        .bind(token)
        .fetch_optional(pool)
        .await?;
        
        Ok(refresh_token)
    }

    async fn delete_refresh_token(&self, token_id: Uuid) -> AuthResult<()> {
        let pool = self.get_pool().await?;
        
        sqlx::query("DELETE FROM refresh_tokens WHERE id = $1")
            .bind(token_id)
            .execute(pool)
            .await?;
        
        Ok(())
    }
}

/// SQL migration script for initializing the database schema.
const INIT_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash TEXT,
    provider VARCHAR(50) NOT NULL,
    provider_id VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT unique_provider_user UNIQUE (provider, provider_id)
);

CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_provider ON users(provider, provider_id);

CREATE TABLE IF NOT EXISTS refresh_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_refresh_tokens_token ON refresh_tokens(token);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_user_id ON refresh_tokens(user_id);
"#;
