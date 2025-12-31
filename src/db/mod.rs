use std::sync::Arc;
use async_trait::async_trait;
use once_cell::sync::OnceCell;
use crate::{error::AuthResult, repo::{provider_repo::ProviderLinkRepo, user_repo::UserRepo}};
use inmemory::InMemoryDatabaseService;

mod inmemory;
mod redis;
mod postgres;


static DB_SERVICE: OnceCell<Arc<dyn DatabaseService>> = OnceCell::new();

#[async_trait]
pub trait DatabaseService: Send + Sync {
	async fn initialize(&self) -> AuthResult<()>;

	fn users(&self) -> Arc<dyn UserRepo>;
	fn providers(&self) -> Arc<dyn ProviderLinkRepo>;
}

/// Get or initialize the global database service.
///
/// This function returns a reference to the global database service singleton.
/// On first call, it initializes the InMemory service and runs migrations.
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
    let service = Arc::new(InMemoryDatabaseService::new()) as Arc<dyn DatabaseService>;

    // Run migrations
    service.initialize().await.expect("Failed to initialize database");

    DB_SERVICE.set(service).unwrap_or_else(|_| panic!("Failed to set database service"));
    DB_SERVICE.get().unwrap()
}
