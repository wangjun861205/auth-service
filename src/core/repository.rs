use crate::core::entities::{CreateUser, User};
use crate::core::error::Error;
pub trait Repository<C> {
    async fn insert_user(&self, user: &CreateUser) -> Result<String, Error>;
    async fn get_password_salt(&self, identifier: &str) -> Result<Option<String>, Error>;
    async fn exists_credential(&self, identifier: &str, password: &str) -> Result<bool, Error>;
    async fn exists_user(&self, identifier: &str) -> Result<bool, Error>;
    async fn set_key(&self, identifier: &str, key: &str) -> Result<(), Error>;
    async fn delete_key(&self, identifier: &str) -> Result<(), Error>;
    async fn get_id_by_key(&self, key: &str) -> Result<Option<String>, Error>;
    async fn generate_claim(&self, identifier: &str) -> Result<C, Error>;
}
