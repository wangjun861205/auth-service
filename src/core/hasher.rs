use crate::core::error::Error;

pub trait Hasher {
    fn generate_salt(&self) -> Result<String, Error>;
    fn hash(&self, content: impl Into<String>, salt: impl Into<String>) -> Result<String, Error>;
}
