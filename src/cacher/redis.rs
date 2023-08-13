use actix_web::{web::Data, FromRequest};
use redis::{aio::Connection, AsyncCommands, Client, ToRedisArgs};
use serde::Serialize;
use serde_json;
use std::fmt::Display;
use std::future::Future;
use std::pin::Pin;

use crate::error::Error;
use crate::services::Cacher;
use crate::services::SecretPair;
use crate::CacherFactory;

pub struct RedisCacherFactory {
    apps_client: Client,
    users_client: Client,
}

impl<ID> CacherFactory<RedisCacher<ID>, ID> for RedisCacherFactory
where
    ID: Default + Clone + Serialize + Display + ToRedisArgs + Send + Sync + 'static,
{
    async fn new_cacher(&self) -> Result<RedisCacher<ID>, Error> {
        let apps_conn = self.apps_client.get_async_connection().await?;
        let users_conn = self.users_client.get_async_connection().await?;
        Ok(RedisCacher {
            apps_conn,
            users_conn,
            phantom: std::marker::PhantomData,
        })
    }
}

impl RedisCacherFactory {
    pub fn new(apps_client: Client, users_client: Client) -> Self {
        Self {
            apps_client,
            users_client,
        }
    }
}

pub struct RedisCacher<ID> {
    apps_conn: Connection,
    users_conn: Connection,
    phantom: std::marker::PhantomData<ID>,
}

impl<ID> Cacher<ID> for RedisCacher<ID>
where
    ID: ToRedisArgs + Send + Sync,
{
    async fn get_app_secret(&mut self, id: ID) -> Result<Option<SecretPair>, Error> {
        if let Some(s) = self.apps_conn.get::<_, Option<String>>(id).await? {
            let pair: SecretPair = serde_json::from_str(&s)?;
            return Ok(Some(pair));
        }
        Ok(None)
    }

    async fn get_user_secret(&mut self, id: ID) -> Result<Option<SecretPair>, Error> {
        if let Some(s) = self.users_conn.get::<_, Option<String>>(id).await? {
            let pair: SecretPair = serde_json::from_str(&s)?;
            return Ok(Some(pair));
        }
        Ok(None)
    }

    async fn put_app_secret(&mut self, id: ID, pair: SecretPair) -> Result<(), Error> {
        let s = serde_json::to_string(&pair)?;
        Ok(self.apps_conn.set(id, s).await?)
    }

    async fn put_user_secret(&mut self, id: ID, pair: SecretPair) -> Result<(), Error> {
        let s = serde_json::to_string(&pair)?;
        Ok(self.users_conn.set(id, s).await?)
    }
}

impl<ID> FromRequest for RedisCacher<ID>
where
    ID: Default + Clone + Serialize + Display + ToRedisArgs + Send + Sync + 'static,
{
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;
    fn extract(req: &actix_web::HttpRequest) -> Self::Future {
        if let Some(factory) = req.app_data::<Data<RedisCacherFactory>>() {
            let factory = factory.clone();
            return Box::pin(async move {
                let cacher =
                    CacherFactory::<RedisCacher<ID>, ID>::new_cacher(factory.as_ref()).await?;
                Ok(cacher)
            });
        }
        Box::pin(async move { Err(Error::CacherError("配置错误".into())) })
    }

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        if let Some(factory) = req.app_data::<Data<RedisCacherFactory>>() {
            let factory: Data<RedisCacherFactory> = factory.clone();
            return Box::pin(async move {
                let cacher =
                    CacherFactory::<RedisCacher<ID>, ID>::new_cacher(factory.as_ref()).await?;
                Ok(cacher)
            });
        }
        Box::pin(async move { Err(Error::CacherError("配置错误".into())) })
    }
}
