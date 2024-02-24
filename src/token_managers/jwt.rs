use crate::core::{error::Error, token_manager::TokenManager};
use jwt::{SignWithKey, SigningAlgorithm, VerifyWithKey, VerifyingAlgorithm};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    async fn sign<C>(&self, claim: C) -> Result<String, Error>
    where
        C: Serialize,
    {
        claim
            .sign_with_key(&self.key)
            .map_err(|e| Error::FailedToSignToken(Box::new(e)))
    }

    async fn verify_token<C>(&self, token: impl Into<String>) -> Result<C, Error>
    where
        for<'de> C: Deserialize<'de>,
    {
        token
            .into()
            .verify_with_key(&self.key)
            .map_err(|e| Error::FailedToVerifyToken(Box::new(e)))
    }
}
