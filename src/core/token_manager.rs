use anyhow::Error;
pub trait TokenManager {
    async fn generate_token(&self, id: impl Into<String>) -> Result<String, Error>;
    async fn verify_token(&self, token: impl Into<String>) -> Result<String, Error>;
}
