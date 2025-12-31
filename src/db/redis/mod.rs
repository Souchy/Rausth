use std::sync::Arc;

use async_trait::async_trait;
use once_cell::sync::{Lazy, OnceCell};

use crate::{
    config::SETTINGS, db::{
        DatabaseService,
        redis::{provider_repo::RedisProviderLinkRepo, user_repo::RedisUserRepo},
    }, error::{AuthError, AuthResult}, repo::{provider_repo::ProviderLinkRepo, user_repo::UserRepo}
};

mod provider_repo;
mod user_repo;

pub struct RedisDatabaseService {
    client: OnceCell<Arc<redis::Client>>,
    user_repo: OnceCell<Arc<RedisUserRepo>>,
    provider_link_repo: OnceCell<Arc<RedisProviderLinkRepo>>,
}

impl RedisDatabaseService {
    pub fn new() -> Self {
        Self {
            client: OnceCell::new(),
            user_repo: OnceCell::new(),
            provider_link_repo: OnceCell::new(),
        }
    }
}

#[async_trait]
impl DatabaseService for RedisDatabaseService {
    async fn initialize(&self) -> AuthResult<()> {
		// create client (sync) and set once
        let client = Arc::new(
            redis::Client::open(SETTINGS.database.url.clone()).expect("Invalid Redis URL")
        );
        
        let conn = client
            .get_multiplexed_async_connection() //.get_async_connection()
            .await
            .map_err(|e| AuthError::DatabaseError(format!("Failed to connect to Redis: {}", e)))?;

        // ignore set error if already initialized
        let _ = self.client.set(client.clone());

        // construct repos now that client exists
        let _ = self.user_repo.set(Arc::new(RedisUserRepo::new(conn.clone())));
        let _ = self.provider_link_repo.set(Arc::new(RedisProviderLinkRepo::new(conn.clone())));

        Ok(())
    }

    fn users(&self) -> Arc<dyn UserRepo> {
        self.user_repo.get().expect("Redis not initialized").clone()
    }

    fn providers(&self) -> Arc<dyn ProviderLinkRepo> {
		self.provider_link_repo.get().expect("Redis not initialized").clone()
    }
}
