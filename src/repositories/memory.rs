use crate::core::{
    entities::{CreateUser, User},
    error::Error,
    repository::Repository,
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

#[derive(Debug, Default, Clone)]
pub struct MemoryRepository {
    map: Arc<RwLock<HashMap<String, User>>>,
}

impl Repository for MemoryRepository {
    async fn exists_user(&self, phone: &str) -> Result<bool, Error> {
        Ok(self.map.read().await.contains_key(phone))
    }

    async fn insert_user(&self, user: &CreateUser) -> Result<String, Error> {
        let mut map = self.map.write().await;
        let id = uuid::Uuid::new_v4().to_string();
        let user = User {
            id: id.clone(),
            phone: user.identifier.clone(),
            password: user.password.clone(),
            password_salt: user.password_salt.clone(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        map.insert(user.phone.clone(), user);
        Ok(id)
    }

    async fn get_password_salt(&self, identifier: &str) -> Result<Option<String>, Error> {
        unimplemented!()
    }

    async fn exists_credential(&self, identifier: &str, password: &str) -> Result<bool, Error> {
        unimplemented!()
    }

    async fn set_key(&self, identifier: &str, key: &str) -> Result<(), Error> {
        unimplemented!()
    }

    async fn get_id_by_key(&self, token: &str) -> Result<Option<String>, Error> {
        unimplemented!()
    }
}
