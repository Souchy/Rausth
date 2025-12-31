use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    db::{
        DatabaseService,
        inmemory::{provider_repo::InMemoryProviderLinkRepo, user_repo::InMemoryUserRepo},
    }, error::AuthResult, repo::{provider_repo::ProviderLinkRepo, user_repo::UserRepo}
};

mod provider_repo;
mod user_repo;

pub struct InMemoryDatabaseService {
    user_repo: Arc<InMemoryUserRepo>,
    provider_link_repo: Arc<InMemoryProviderLinkRepo>,
}

impl InMemoryDatabaseService {
    pub fn new() -> Self {
        Self {
            user_repo: Arc::new(InMemoryUserRepo::new()),
            provider_link_repo: Arc::new(InMemoryProviderLinkRepo::new()),
        }
    }
}

#[async_trait]
impl DatabaseService for InMemoryDatabaseService {
    async fn initialize(&self) -> AuthResult<()> {
        // No initialization needed for in-memory database
        Ok(())
    }

    fn users(&self) -> Arc<dyn UserRepo> {
        self.user_repo.clone()
    }

    fn providers(&self) -> Arc<dyn ProviderLinkRepo> {
        self.provider_link_repo.clone()
    }
}
