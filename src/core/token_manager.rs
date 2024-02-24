use serde::{Deserialize, Serialize};

use crate::core::error::Error;

pub trait TokenManager {
    async fn sign<C>(&self, claim: C) -> Result<String, Error>
    where
        C: Serialize;
    async fn verify_token<C>(&self, token: impl Into<String>) -> Result<C, Error>
    where
        for<'de> C: Deserialize<'de>;
}
