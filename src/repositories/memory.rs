use crate::core::{
    entities::{CreateUser, User},
    repository::Repository,
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

#[derive(Debug, Default, Clone)]
pub struct MemoryRepository {
    map: Arc<RwLock<HashMap<String, User>>>,
}

impl Repository for MemoryRepository {
    async fn exists_user(&self, phone: &str) -> Result<bool, anyhow::Error> {
        Ok(self.map.read().await.contains_key(phone))
    }

    async fn fetch_user(&self, phone: &str) -> Result<Option<User>, anyhow::Error> {
        Ok(self.map.read().await.get(phone).cloned())
    }
    async fn insert_user(&self, user: &CreateUser) -> Result<String, anyhow::Error> {
        let mut map = self.map.write().await;
        let id = uuid::Uuid::new_v4().to_string();
        let user = User {
            id: id.clone(),
            phone: user.phone.clone(),
            password: user.password.clone(),
            password_salt: user.password_salt.clone(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        map.insert(user.phone.clone(), user);
        Ok(id)
    }
}
