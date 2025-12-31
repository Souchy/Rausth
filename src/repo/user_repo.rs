use async_trait::async_trait;

#[derive(Clone)]
pub struct User {
	pub id: String,
	pub email: String,
	pub name: String,
	// Add other relevant fields
}

#[async_trait]
pub trait UserRepo: Send + Sync {
    async fn add_user(&self, user: User) -> Result<(), String>;
    async fn get_user_by_email(&self, email: &str) -> Result<User, String>;
    async fn get_user_by_id(&self, id: &str) -> Result<User, String>;
}
