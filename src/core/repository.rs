use crate::core::entities::CreateUser;
use crate::core::error::Error;
pub trait Repository<C> {
    async fn insert_user(&self, user: &CreateUser) -> Result<String, Error>;
    async fn get_password_salt(&self, identifier: &str) -> Result<Option<String>, Error>;
    async fn exists_credential(&self, identifier: &str, password: &str) -> Result<bool, Error>;
    async fn exists_user(&self, identifier: &str) -> Result<bool, Error>;
    async fn generate_claim(&self, identifier: &str) -> Result<C, Error>;
    // 更新identifier对应的token， 用于单设备登录，如果允许多设备登录，可使用默认实现
    async fn update_token(&self, identifier: &str, token: &str) -> Result<(), Error> {
        Ok(())
    }
    // token是否存在(是否当前有效)， 用于单设备登录，如果允许多设备登录，可使用默认实现
    async fn exists_token(&self, token: &str) -> Result<bool, Error> {
        Ok(true)
    }
}
