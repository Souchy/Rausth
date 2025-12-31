use std::{collections::HashMap, sync::RwLock};

use async_trait::async_trait;
use once_cell::sync::Lazy;

use crate::repo::user_repo::{User, UserRepo};


// In Memory User Repository implementation for local development
pub struct InMemoryUserRepo {
    users: Lazy<RwLock<HashMap<String, User>>>,
}

impl InMemoryUserRepo {
    pub fn new() -> Self {
        Self {
            users: Lazy::new(|| RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl UserRepo for InMemoryUserRepo {
    async fn add_user(&self, user: User) -> Result<(), String> {
        let mut users = self.users.write().unwrap();
        if users.contains_key(&user.email) {
            return Err(format!("User with email {} already exists!", user.email));
        }
        users.insert(user.email.clone(), user);
        Ok(())
    }

    async fn get_user_by_email(&self, email: &str) -> Result<User, String> {
        let users = self.users.read().unwrap();
        match users.get(email) {
            Some(user) => Ok(user.clone()),
            None => Err(format!("User with email {} not found!", email)),
        }
    }

    async fn get_user_by_id(&self, id: &str) -> Result<User, String> {
        let users = self.users.read().unwrap();
        for user in users.values() {
            if user.id == id {
                return Ok(user.clone());
            }
        }
        Err(format!("User with id {} not found!", id))
    }
}
