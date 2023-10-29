use crate::core::entities::{CreateUser, User};
use anyhow::Error;
pub trait Repository {
    async fn insert_user(&self, user: &CreateUser) -> Result<String, Error>;
    async fn fetch_user(&self, phone: impl AsRef<String>) -> Result<Option<User>, Error>;
    async fn exists_user(&self, phone: impl AsRef<String>) -> Result<bool, Error>;
}
