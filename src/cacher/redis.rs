use crate::CacherFactory;
use actix_web::{web::Data, FromRequest};
use redis::{aio::Connection, AsyncCommands, Client, ToRedisArgs};
use serde::Serialize;
use std::fmt::Display;
use std::future::Future;
use std::pin::Pin;

use crate::error::Error;
use crate::services::Cacher;
use crate::services::SecretPair;

pub struct RedisCacherFactory {
    apps_client: Client,
    users_client: Client,
}

impl<ID> CacherFactory<RedisCacher, ID> for RedisCacherFactory
where
    ID: Default + Clone + Serialize + Display + ToRedisArgs + Send + Sync,
{
    async fn new_cacher(&self) -> Result<RedisCacher, Error> {
        let apps_conn = self.apps_client.get_async_connection().await?;
        let users_conn = self.users_client.get_async_connection().await?;
        Ok(RedisCacher {
            apps_conn,
            users_conn,
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

pub struct RedisCacher {
    apps_conn: Connection,
    users_conn: Connection,
}

impl<ID> Cacher<ID> for RedisCacher
where
    ID: ToRedisArgs + Send + Sync,
{
    async fn get_app_secret(&mut self, id: ID) -> Result<Option<SecretPair>, Error> {
        let pair: Option<SecretPair> = self.apps_conn.get(id).await?;
        Ok(pair)
    }

    async fn get_user_secret(&mut self, id: ID) -> Result<Option<SecretPair>, Error> {
        let pair: Option<SecretPair> = self.users_conn.get(id).await?;
        Ok(pair)
    }

    async fn put_app_secret(&mut self, id: ID, pair: SecretPair) -> Result<(), Error> {
        Ok(self.apps_conn.set(id, pair).await?)
    }

    async fn put_user_secret(&mut self, id: ID, pair: SecretPair) -> Result<(), Error> {
        Ok(self.users_conn.set(id, pair).await?)
    }
}

impl FromRequest for RedisCacher {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;
    fn extract(req: &actix_web::HttpRequest) -> Self::Future {
        if let Some(factory) = req.app_data::<Data<RedisCacherFactory>>() {
            let factory = factory.clone();
            return Box::pin(async move {
                let cacher =
                    CacherFactory::<RedisCacher, String>::new_cacher(factory.as_ref()).await?;
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
                    CacherFactory::<RedisCacher, String>::new_cacher(factory.as_ref()).await?;
                Ok(cacher)
            });
        }
        Box::pin(async move { Err(Error::CacherError("配置错误".into())) })
    }
}
