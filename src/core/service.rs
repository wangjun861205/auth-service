use anyhow::Error;

use crate::core::{
    entities::CreateUser, hasher::Hasher, repository::Repository, token_manager::TokenManager,
};

#[derive(Debug, Clone)]
pub struct Service<R, H, T>
where
    R: Repository + Clone,
    H: Hasher + Clone,
    T: TokenManager + Clone,
{
    pub repository: R,
    pub hasher: H,
    pub token_manager: T,
}

impl<R, H, T> Service<R, H, T>
where
    R: Repository + Clone,
    H: Hasher + Clone,
    T: TokenManager + Clone,
{
    pub fn new(repository: R, hasher: H, token_manager: T) -> Self {
        Self {
            repository,
            hasher,
            token_manager,
        }
    }

    pub async fn signup(&self, phone: &str, password: &str) -> Result<String, Error> {
        if self.repository.exists_user(phone).await? {
            return Err(Error::msg("手机号已被注册"));
        }

        let password_salt = self.hasher.generate_salt()?;
        let hashed_password = self.hasher.hash(password, &password_salt)?;
        let id = self
            .repository
            .insert_user(&CreateUser {
                phone: phone.to_owned(),
                password: hashed_password,
                password_salt,
            })
            .await?;
        self.token_manager.generate_token(&id).await
    }

    pub async fn generate_token(&self, phone: &str) -> Result<String, Error> {
        let user = self
            .repository
            .fetch_user(phone)
            .await?
            .ok_or(Error::msg("用户不存在"))?;
        self.token_manager.generate_token(&user.id).await
    }

    pub async fn verify_token(&self, token: &str) -> Result<String, Error> {
        self.token_manager.verify_token(token).await
    }

    pub async fn login_by_password(&self, phone: &str, password: &str) -> Result<String, Error> {
        if let Some(user) = self.repository.fetch_user(phone).await? {
            if self.hasher.hash(password, user.password_salt)? != user.password {
                return Err(Error::msg("用户不存在或凭证不正确"));
            }
            return self.token_manager.generate_token(&user.id).await;
        }
        Err(Error::msg("用户不存在或凭证不正确"))
    }

    pub async fn exists_user(&self, phone: &str) -> Result<bool, Error> {
        self.repository.exists_user(phone).await
    }
}
