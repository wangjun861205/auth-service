use crate::error::Error;
use actix_web::FromRequest;
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

use crate::services::SecretGenerator;

pub struct RandomSecretGenerator;

impl SecretGenerator for RandomSecretGenerator {
    fn generate_secret(&self) -> Result<String, crate::error::Error> {
        Ok(Uuid::new_v4().to_string())
    }
}

impl FromRequest for RandomSecretGenerator {
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;
    type Error = Error;
    fn extract(req: &actix_web::HttpRequest) -> Self::Future {
        Box::pin(async { Ok(RandomSecretGenerator) })
    }

    fn from_request(
        req: &actix_web::HttpRequest,
        payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        Box::pin(async { Ok(RandomSecretGenerator) })
    }
}
