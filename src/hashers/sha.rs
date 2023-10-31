use crate::core::hasher::Hasher;
use anyhow::Error;

use hex::encode;
use sha2::{Digest, Sha384};
use uuid::Uuid;

#[derive(Debug, Default, Clone)]
pub struct ShaHasher;

impl Hasher for ShaHasher {
    fn generate_salt(&self) -> Result<String, Error> {
        Ok(Uuid::new_v4().to_string())
    }
    fn hash(&self, content: impl Into<String>, salt: impl Into<String>) -> Result<String, Error> {
        let mut hasher = Sha384::new();
        hasher.update(content.into());
        hasher.update(salt.into());
        let result = hasher.finalize();
        Ok(encode(result))
    }
}
