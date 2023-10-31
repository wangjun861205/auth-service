use crate::core::token_manager::TokenManager;
use anyhow::Error;
use jwt::{SignWithKey, SigningAlgorithm, VerifyWithKey, VerifyingAlgorithm};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    id: String,
}

#[derive(Debug, Default, Clone)]
pub struct JWTTokenManager<T>
where
    T: SigningAlgorithm,
{
    key: T,
}

impl<T> JWTTokenManager<T>
where
    T: SigningAlgorithm,
{
    pub fn new(key: T) -> Self {
        Self { key }
    }
}

impl<T> TokenManager for JWTTokenManager<T>
where
    T: SigningAlgorithm + VerifyingAlgorithm,
{
    async fn generate_token(&self, id: impl Into<String>) -> Result<String, Error> {
        let claims = Claims { id: id.into() };
        let token = claims.sign_with_key(&self.key)?;
        Ok(token.to_string())
    }

    async fn verify_token(&self, token: impl Into<String>) -> Result<String, Error> {
        let claims: Claims = token.into().verify_with_key(&self.key)?;
        Ok(claims.id)
    }
}
