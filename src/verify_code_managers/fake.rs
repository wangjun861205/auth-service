use actix_web::FromRequest;
use rand::{
    distributions::{Alphanumeric, DistString},
    thread_rng, Rng,
};
use std::{collections::HashMap, future::ready};

use crate::error::Error;
use crate::services::VerifyCodeManager;
use std::sync::{Arc, Mutex};

pub type Map = Arc<Mutex<HashMap<String, String>>>;

pub struct FakeVerifyCodeManager {
    email: Map,
    phone: Map,
}

impl VerifyCodeManager for FakeVerifyCodeManager {
    async fn send_by_email(&mut self, email: impl Into<String>) -> Result<(), Error> {
        let email = email.into();
        println!("email: {}", &email);
        let code = Alphanumeric.sample_string(&mut thread_rng(), 6);
        self.email.lock().unwrap().insert(email, code);
        Ok(())
    }

    async fn send_by_sms(&mut self, phone: impl Into<String>) -> Result<(), Error> {
        let phone = phone.into();
        println!("email: {}", &phone);
        let code = Alphanumeric.sample_string(&mut thread_rng(), 6);
        self.phone.lock().unwrap().insert(phone, code);
        Ok(())
    }

    async fn verify_email_code(
        &mut self,
        email: impl Into<String>,
        code: impl Into<String>,
    ) -> Result<(), Error> {
        if let Some(c) = self.email.lock().unwrap().get(&email.into()) {
            if c == &code.into() {
                return Ok(());
            }
        }
        Err(Error::ServiceError("验证码不正确".into()))
    }

    async fn verify_sms_code(
        &mut self,
        phone: impl Into<String>,
        code: impl Into<String>,
    ) -> Result<(), Error> {
        if let Some(c) = self.phone.lock().unwrap().get(&phone.into()) {
            if c == &code.into() {
                return Ok(());
            }
        }
        Err(Error::ServiceError("验证码不正确".into()))
    }
}

impl FromRequest for FakeVerifyCodeManager {
    type Error = Error;
    type Future = std::future::Ready<Result<Self, Self::Error>>;

    fn extract(req: &actix_web::HttpRequest) -> Self::Future {
        if let Some(tuple) = req.app_data::<(Map, Map)>() {
            return ready(Ok(Self {
                email: tuple.0.clone(),
                phone: tuple.1.clone(),
            }));
        }
        ready(Err(Error::ServiceError("配置错误".into())))
    }

    fn from_request(
        req: &actix_web::HttpRequest,
        payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        if let Some(tuple) = req.app_data::<(Map, Map)>() {
            return ready(Ok(Self {
                email: tuple.0.clone(),
                phone: tuple.1.clone(),
            }));
        }
        ready(Err(Error::ServiceError("配置错误".into())))
    }
}
