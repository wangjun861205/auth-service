use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::core::{
    entities::CreateUser, error::Error, hasher::Hasher, repository::Repository,
    token_manager::TokenManager,
};

#[derive(Debug, Clone)]
pub struct Service<R, H, T, C>
where
    R: Repository<C> + Clone,
    H: Hasher + Clone,
    T: TokenManager + Clone,
    for<'de> C: Serialize + Deserialize<'de>,
{
    pub repository: R,
    pub hasher: H,
    pub token_manager: T,
    pub single_device: bool, // 是否是单设备登录模式, 开启后同一时间只允许同一设备登录， 其他设备登录会使当前设备的登录失效
    _phantom: PhantomData<C>,
}

impl<R, H, T, C> Service<R, H, T, C>
where
    R: Repository<C> + Clone,
    H: Hasher + Clone,
    T: TokenManager + Clone,
    for<'de> C: Serialize + Deserialize<'de>,
{
    pub fn new(repository: R, hasher: H, token_manager: T, single_device: bool) -> Self {
        Self {
            repository,
            hasher,
            token_manager,
            single_device,
            _phantom: PhantomData,
        }
    }

    pub async fn signup(&self, identifier: &str, password: &str) -> Result<String, Error> {
        if self.repository.exists_user(identifier).await? {
            return Err(Error::IdentifierAlreadyExists);
        }
        let password_salt = self.hasher.generate_salt()?;
        let hashed_password = self.hasher.hash(password, &password_salt)?;
        let id = self
            .repository
            .insert_user(&CreateUser {
                identifier: identifier.to_owned(),
                password: hashed_password,
                password_salt,
            })
            .await?;
        Ok(id)
    }

    pub async fn generate_token(&self, claim: C) -> Result<String, Error> {
        self.token_manager
            .sign(claim)
            .await
            .map_err(|e| Error::TokenManagerError(Box::new(e)))
    }

    pub async fn verify_token(&self, token: &str) -> Result<C, Error> {
        let claim = self
            .token_manager
            .verify_token(token)
            .await
            .map_err(|e| Error::TokenManagerError(Box::new(e)))?;
        if self.single_device && !self.repository.exists_token(token).await? {
            return Err(Error::InvalidToken);
        }
        Ok(claim)
    }

    pub async fn login(&self, identifier: &str, password: &str) -> Result<String, Error> {
        if let Some(salt) = self
            .repository
            .get_password_salt(identifier)
            .await
            .map_err(|e| Error::RepositoryError(Box::new(e)))?
        {
            let hashed_password = self
                .hasher
                .hash(password, &salt)
                .map_err(|e| Error::HasherError(Box::new(e)))?;
            if self
                .repository
                .exists_credential(identifier, &hashed_password)
                .await
                .map_err(|e| Error::RepositoryError(Box::new(e)))?
            {
                let claim = self
                    .repository
                    .generate_claim(identifier)
                    .await
                    .map_err(|e| Error::FailedToGenerateClaim(Box::new(e)))?;
                let token = self.generate_token(claim).await?;
                if self.single_device {
                    self.repository.update_token(identifier, &token).await?;
                }
                return Ok(token);
            }
        }
        Err(Error::InvalidCredential)
    }

    pub async fn exists_user(&self, identifier: &str) -> Result<bool, Error> {
        self.repository.exists_user(identifier).await
    }
}
