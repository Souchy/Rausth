mod service;
mod postgres;

pub use service::DatabaseService;
pub use postgres::PostgresService;

use once_cell::sync::OnceCell;
use std::sync::Arc;

/// Global database service instance.
///
/// This singleton is initialized once and reused throughout the application.
static DB_SERVICE: OnceCell<Arc<dyn DatabaseService>> = OnceCell::new();

/// Get or initialize the global database service.
///
/// This function returns a reference to the global database service singleton.
/// On first call, it initializes the PostgreSQL service and runs migrations.
///
/// # Returns
/// A reference to the database service implementation
///
/// # Panics
/// Panics if database initialization fails
pub async fn get_db_service() -> &'static Arc<dyn DatabaseService> {
    if let Some(service) = DB_SERVICE.get() {
        return service;
    }

    // Initialize the database service
    let service = Arc::new(PostgresService::new()) as Arc<dyn DatabaseService>;
    
    // Run migrations
    service.initialize().await.expect("Failed to initialize database");
    
    DB_SERVICE.set(service).unwrap_or_else(|_| panic!("Failed to set database service"));
    DB_SERVICE.get().unwrap()
}

// Legacy function for backward compatibility
// TODO: Remove this once all code is migrated to use DatabaseService trait
pub async fn get_pool() -> Result<&'static sqlx::PgPool, sqlx::Error> {
    use once_cell::sync::OnceCell;
    use sqlx::{PgPool, postgres::PgPoolOptions};
    
    static DB_POOL: OnceCell<PgPool> = OnceCell::new();
    
    if let Some(pool) = DB_POOL.get() {
        return Ok(pool);
    }

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    
    // Run migrations
    sqlx::query(migrations::INIT_SQL)
        .execute(&pool)
        .await?;
    
    DB_POOL.set(pool).map_err(|_| sqlx::Error::PoolClosed)?;
    Ok(DB_POOL.get().unwrap())
}

pub mod migrations {
    pub const INIT_SQL: &str = r#"
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
}
