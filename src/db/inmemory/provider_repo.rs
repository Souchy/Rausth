use std::{collections::HashMap, sync::RwLock};

use async_trait::async_trait;
use once_cell::sync::Lazy;

use crate::repo::{provider_repo::{ProviderLink, ProviderLinkRepo}, user_repo::User};

// In Memory ProviderLink Repository implementation for local development
pub struct InMemoryProviderLinkRepo {
    // Organize ProviderLinks by user_id, then by provider_account_id
    provider_links: Lazy<RwLock<HashMap<String, HashMap<String, ProviderLink>>>>,
}

impl InMemoryProviderLinkRepo {
    pub fn new() -> Self {
        Self {
            provider_links: Lazy::new(|| RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl ProviderLinkRepo for InMemoryProviderLinkRepo {
    async fn add_provider_link(&self, provider_link: ProviderLink) -> Result<(), String> {
        let mut links = self.provider_links.write().unwrap();
        let user_links = links.entry(provider_link.user_id.clone()).or_insert_with(HashMap::new);
        if user_links.contains_key(&provider_link.provider_account_id) {
            return Err(format!("ProviderLink with provider_account_id {} already exists for user {}!", provider_link.provider_account_id, provider_link.user_id));
        }
        user_links.insert(provider_link.provider_account_id.clone(), provider_link);
        Ok(())
    }
	async fn get_provider_link_by_id(&self, user: &User, provider_account_id: &str) -> Result<ProviderLink, String> {
        let links = self.provider_links.read().unwrap();
        if let Some(user_links) = links.get(&user.id) {
            if let Some(provider_link) = user_links.get(provider_account_id) {
                return Ok(provider_link.clone());
            }
        }
        Err(format!("ProviderLink with provider_account_id {} not found for user {}!", provider_account_id, user.id))
	}
}
