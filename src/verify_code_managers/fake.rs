use actix_web::{web::Data, FromRequest};
use rand::{
    distributions::{Alphanumeric, DistString},
    thread_rng,
};
use std::collections::HashMap;

use crate::services::VerifyCodeManager;
use crate::{error::Error, VerifyCodeManagerFactory};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

pub struct FakeVerifyCodeManagerFactory {
    email: Map,
    phone: Map,
}

impl FakeVerifyCodeManagerFactory {
    pub fn new(
        email: Arc<Mutex<HashMap<String, String>>>,
        phone: Arc<Mutex<HashMap<String, String>>>,
    ) -> Self {
        Self { email, phone }
    }
}
impl VerifyCodeManagerFactory<FakeVerifyCodeManager> for FakeVerifyCodeManagerFactory {
    async fn new_verify_code_manager(&self) -> Result<FakeVerifyCodeManager, crate::error::Error> {
        Ok(FakeVerifyCodeManager {
            email: self.email.clone(),
            phone: self.phone.clone(),
        })
    }
}

pub type Map = Arc<Mutex<HashMap<String, String>>>;

pub struct FakeVerifyCodeManager {
    email: Map,
    phone: Map,
}

impl VerifyCodeManager for FakeVerifyCodeManager {
    async fn send_by_email(&mut self, email: impl Into<String>) -> Result<(), Error> {
        let email = email.into();
        let code = Alphanumeric.sample_string(&mut thread_rng(), 6);
        println!("email: {}, code: {}", &email, &code);
        self.email.lock().unwrap().insert(email, code);
        Ok(())
    }

    async fn send_by_sms(&mut self, phone: impl Into<String>) -> Result<(), Error> {
        let phone = phone.into();
        let code = Alphanumeric.sample_string(&mut thread_rng(), 6);
        println!("phone: {}, code: {}", &phone, &code);
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
        for (k, v) in self.phone.lock().unwrap().iter() {
            println!("phone: {}, code: {}", k, v);
        }
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
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn extract(req: &actix_web::HttpRequest) -> Self::Future {
        if let Some(factory) = req.app_data::<Data<FakeVerifyCodeManagerFactory>>() {
            let factory = factory.clone();
            return Box::pin(async move {
                let manager = factory.new_verify_code_manager().await?;
                Ok(manager)
            });
        }
        Box::pin(async move { Err(Error::ServiceError("配置错误".into())) })
    }

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        if let Some(factory) = req.app_data::<Data<FakeVerifyCodeManagerFactory>>() {
            let factory = factory.clone();
            return Box::pin(async move {
                let manager = factory.new_verify_code_manager().await?;
                Ok(manager)
            });
        }
        Box::pin(async move { Err(Error::ServiceError("配置错误".into())) })
    }
}
