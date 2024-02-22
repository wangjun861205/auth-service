use crate::core::{
    entities::CreateUser, error::Error, hasher::Hasher, repository::Repository,
    token_manager::TokenManager,
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

    pub async fn generate_token(&self, identifier: &str) -> Result<String, Error> {
        let key = self
            .token_manager
            .generate_key()
            .await
            .map_err(|e| Error::TokenManagerError(Box::new(e)))?;
        let token = self
            .token_manager
            .sign_key(&key)
            .await
            .map_err(|e| Error::TokenManagerError(Box::new(e)))?;
        self.repository
            .set_key(identifier, &key)
            .await
            .map_err(|e| Error::RepositoryError(Box::new(e)))?;
        Ok(token)
    }

    pub async fn verify_token(&self, token: &str) -> Result<String, Error> {
        let key = self
            .token_manager
            .verify_token(token)
            .await
            .map_err(|e| Error::TokenManagerError(Box::new(e)))?;
        let id = self
            .repository
            .get_id_by_key(&key)
            .await
            .map_err(|e| Error::RepositoryError(Box::new(e)))?
            .ok_or(Error::InvalidToken)?;
        Ok(id)
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
                return self.generate_token(identifier).await;
            }
        }
        Err(Error::InvalidCredential)
    }

    pub async fn logout(&self, identifier: &str) -> Result<(), Error> {
        self.repository.delete_key(identifier).await
    }

    pub async fn exists_user(&self, identifier: &str) -> Result<bool, Error> {
        self.repository.exists_user(identifier).await
    }
}
