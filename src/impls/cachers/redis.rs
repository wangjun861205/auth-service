use redis::{AsyncCommands, Client, ToRedisArgs};
use std::error::Error as StdErr;
use std::marker::PhantomData;

use crate::core::cacher::{Cacher, SecretPair};

#[derive(Debug, Clone)]
pub struct RedisCacher<ID> {
    client: Client,
    phantom: PhantomData<ID>,
}

impl<ID> RedisCacher<ID> {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            phantom: PhantomData,
        }
    }
}

impl<ID> Cacher<ID> for RedisCacher<ID>
where
    ID: ToRedisArgs + Send + Sync,
{
    async fn get_user_secret(&mut self, id: ID) -> Result<Option<SecretPair>, Box<dyn StdErr>> {
        if let Some(s) = self
            .client
            .get_async_connection()
            .await?
            .get::<_, Option<String>>(id)
            .await?
        {
            let pair: SecretPair = serde_json::from_str(&s)?;
            return Ok(Some(pair));
        }
        Ok(None)
    }

    async fn put_user_secret(&mut self, id: ID, pair: SecretPair) -> Result<(), Box<dyn StdErr>> {
        let s = serde_json::to_string(&pair)?;
        Ok(self.client.get_async_connection().await?.set(id, s).await?)
    }
}
