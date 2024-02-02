use crate::core::entities::{CreateUser, User};
use crate::core::error::Error;
pub trait Repository {
    async fn insert_user(&self, user: &CreateUser) -> Result<String, Error>;
    async fn get_password_salt(&self, identifier: &str) -> Result<Option<String>, Error>;
    async fn get_id_by_credential(
        &self,
        identifier: &str,
        password: &str,
    ) -> Result<Option<String>, Error>;
    async fn exists_user(&self, identifier: &str) -> Result<bool, Error>;
    async fn set_token(&self, identifier: &str, token: &str) -> Result<(), Error>;
    async fn get_id_by_key(&self, key: &str) -> Result<Option<String>, Error>;
}
