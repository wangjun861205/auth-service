use crate::core::entities::CreateUser;
use crate::core::error::Error;
pub trait Repository<C> {
    async fn insert_user(&self, user: &CreateUser) -> Result<String, Error>;
    async fn get_password_salt(&self, identifier: &str) -> Result<Option<String>, Error>;
    async fn exists_credential(&self, identifier: &str, password: &str) -> Result<bool, Error>;
    async fn exists_user(&self, identifier: &str) -> Result<bool, Error>;
    async fn generate_claim(&self, identifier: &str) -> Result<C, Error>;
}
