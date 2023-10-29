use std::error::Error as StdErr;
pub trait TokenManager {
    fn generate_token(&self, id: impl AsRef<String>) -> Result<String, Box<dyn StdErr>>;
    fn verify_token(&self, token: impl AsRef<String>) -> Result<String, Box<dyn StdErr>>;
}
