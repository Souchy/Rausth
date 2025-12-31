use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use redis::{AsyncCommands, aio::MultiplexedConnection};

use crate::repo::{
    provider_repo::{ProviderLink, ProviderLinkRepo},
    user_repo::User,
};

// Redis ProviderLink Repository implementation
pub struct RedisProviderLinkRepo {
    conn: MultiplexedConnection,
}

impl RedisProviderLinkRepo {
    pub fn new(conn: MultiplexedConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl ProviderLinkRepo for RedisProviderLinkRepo {
    async fn add_provider_link(&self, provider_link: ProviderLink) -> Result<(), String> {
        let mut conn = self.conn.clone();

        let key = format!("user:{}:provider_account_id:{}", provider_link.user_id, provider_link.provider_account_id);
        let _: () = conn
            .hset_multiple(
                &key,
                &[
                    // ("id", &provider_link.id),
                    ("user_id", &provider_link.user_id),
                    ("provider_account_id", &provider_link.provider_account_id),
                    ("email", &provider_link.email),
                    ("access_token", &provider_link.access_token),
                    ("refresh_token", &provider_link.refresh_token),
                    ("provider", &provider_link.provider),
                ],
            )
            .await
            .map_err(|e| format!("Failed to add provider link to Redis: {}", e))?;

        Ok(())
    }
    
    async fn get_provider_link_by_id(
        &self,
        user: &User,
        provider_account_id: &str,
    ) -> Result<ProviderLink, String> {

        let mut conn = self.conn.clone();

        let key = format!("user:{}:provider_account_id:{}", user.id, provider_account_id);
        let result: HashMap<String, String> = conn
            .hgetall(&key)
            .await
            .map_err(|e| format!("Failed to get provider link from Redis: {}", e))?;

        if result.is_empty() {
            return Err(format!(
                "Provider link with id {} not found for user {}",
                provider_account_id, user.id
            ));
        }

        Ok(ProviderLink {
            // id: result.get("id").cloned().unwrap_or_default(),
            user_id: result.get("user_id").cloned().unwrap_or_default(),
            provider_account_id: result.get("provider_account_id").cloned().unwrap_or_default(),
            email: result.get("email").cloned().unwrap_or_default(),
            access_token: result.get("access_token").cloned().unwrap_or_default(),
            refresh_token: result.get("refresh_token").cloned().unwrap_or_default(),
            provider: result.get("provider").cloned().unwrap_or_default(),
        })

    }
}
