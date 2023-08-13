use crate::error::Error;
use std::future::Future;
use std::pin::Pin;

use crate::{services::Hasher, HasherFactory};
use actix_web::{web::Data, FromRequest};
use hex::encode;
use sha2::{Digest, Sha384};
use uuid::Uuid;

pub struct ShaHasherFactory;

impl ShaHasherFactory {
    pub fn new() -> Self {
        Self {}
    }
}

impl HasherFactory<ShaHasher> for ShaHasherFactory {
    async fn new_hasher(&self) -> Result<ShaHasher, Error> {
        Ok(ShaHasher {})
    }
}
pub struct ShaHasher;

impl Hasher for ShaHasher {
    fn generate_salt(&self) -> Result<String, crate::error::Error> {
        Ok(Uuid::new_v4().to_string())
    }
    fn hash(
        &self,
        content: impl Into<String>,
        salt: impl Into<String>,
    ) -> Result<String, crate::error::Error> {
        let mut hasher = Sha384::new();
        hasher.update(content.into());
        hasher.update(salt.into());
        let result = hasher.finalize();
        Ok(encode(result))
    }
}

impl FromRequest for ShaHasher {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<ShaHasher, Error>>>>;

    fn extract(req: &actix_web::HttpRequest) -> Self::Future {
        if let Some(factory) = req.app_data::<Data<ShaHasherFactory>>() {
            let factory = factory.clone();
            return Box::pin(async move { factory.new_hasher().await });
        }
        Box::pin(async move { Err(Error::ServerError("Hasher配置错误".into())) })
    }

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        if let Some(factory) = req.app_data::<Data<ShaHasherFactory>>() {
            let factory = factory.clone();
            return Box::pin(async move { factory.new_hasher().await });
        }
        Box::pin(async move { Err(Error::ServerError("Hasher配置错误".into())) })
    }
}
