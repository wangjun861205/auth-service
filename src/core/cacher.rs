use serde::{Deserialize, Serialize};
use std::{error::Error as StdErr, fmt::Debug};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SecretPair {
    pub hashed_secret: String,
    pub secret_salt: String,
}

pub trait Cacher<ID> {
    async fn put_user_secret(&mut self, id: ID, secret: SecretPair) -> Result<(), Box<dyn StdErr>>;
    async fn get_user_secret(&mut self, id: ID) -> Result<Option<SecretPair>, Box<dyn StdErr>>;
}
