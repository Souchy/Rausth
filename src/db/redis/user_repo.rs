use std::collections::HashMap;

use async_trait::async_trait;
use redis::{AsyncCommands, aio::MultiplexedConnection};

use crate::repo::user_repo::{User, UserRepo};

// Redis User Repository implementation for local development
pub struct RedisUserRepo {
    conn: MultiplexedConnection,
}

impl RedisUserRepo {
    pub fn new(conn: MultiplexedConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl UserRepo for RedisUserRepo {
    async fn add_user(&self, user: User) -> Result<(), String> {
        let mut conn = self.conn.clone();
        let key = format!("user:{}", user.id);
        let _: () = conn
            .hset_multiple(
                &key,
                &[
                    ("id", &user.id),
                    ("email", &user.email),
                    ("name", &user.name),
                    ("access_token", &user.access_token),
                    ("refresh_token", &user.refresh_token),
                ],
            )
            .await
            .map_err(|e| format!("Failed to add user to Redis: {}", e))?;

        Ok(())
    }

    async fn get_user_by_email(&self, email: &str) -> Result<User, String> {
        let mut conn = self.conn.clone();
        let mut conn_iter = self.conn.clone();

        // Scan through keys to find user by email
        let mut iter: redis::AsyncIter<String> = conn
            .scan_match("user:*")
            .await
            .map_err(|e| format!("Failed to scan users in Redis: {}", e))?;

        while let Some(key) = iter.next_item().await {
            let user_email: String = conn_iter
                .hget(&key, "email")
                .await
                .map_err(|e| format!("Failed to get email from Redis: {}", e))?;
            if user_email == email {
                 // key is "user:{id}" — strip prefix to get id
                let id = key
                    .strip_prefix("user:")
                    .map(|s| s.to_string())
                    .unwrap_or(key.clone());
                return self.get_user_by_id(&id).await;
            }
        }

        Err(format!("User with email {} not found", email))
    }

    async fn get_user_by_id(&self, id: &str) -> Result<User, String> {
        let mut conn = self.conn.clone();
        let key = format!("user:{}", id);
        let result: HashMap<String, String> = conn
            .hgetall(&key)
            .await
            .map_err(|e| format!("Failed to get user from Redis: {}", e))?;

        if result.is_empty() {
            return Err(format!("User with id {} not found", id));
        }

        Ok(User {
            id: result.get("id").cloned().unwrap_or_default(),
            email: result.get("email").cloned().unwrap_or_default(),
            name: result.get("name").cloned().unwrap_or_default(),
            access_token: result.get("access_token").cloned().unwrap_or_default(),
            refresh_token: result.get("refresh_token").cloned().unwrap_or_default(),
        })
    }
}
