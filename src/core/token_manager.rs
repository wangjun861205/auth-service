use crate::core::error::Error;

pub trait TokenManager {
    async fn generate_key(&self) -> Result<impl Into<String>, Error>;
    async fn sign_key(&self, id: impl Into<String>) -> Result<String, Error>;
    async fn verify_token(&self, token: impl Into<String>) -> Result<String, Error>;
}
