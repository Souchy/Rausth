use async_trait::async_trait;

use crate::repo::user_repo::User;

#[derive(Clone)]
pub struct ProviderLink {
	// Define the fields for ProviderLink here
	// pub id: String,
	pub user_id: String,
	pub provider_account_id: String,
	pub email: String,
	pub access_token: String,
	pub refresh_token: String,
	pub provider: String,
	// pub scopes: Vec<String>, // ?
	// Add other relevant fields
}

#[async_trait]
pub trait ProviderLinkRepo: Send + Sync {
	// Define the methods for ProviderLinkRepo here
	async fn add_provider_link(&self, link: ProviderLink) -> Result<(), String>;
	async fn get_provider_link_by_id(&self, user: &User, provider_account_id: &str) -> Result<ProviderLink, String>;
}
