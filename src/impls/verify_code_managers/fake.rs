use rand::{
    distributions::{Alphanumeric, DistString},
    thread_rng,
};
use std::{collections::HashMap, error::Error as StdErr};

use crate::core::{error::Error, verify_code_manager::VerifyCodeManager};
use std::sync::{Arc, Mutex};

pub type Map = Arc<Mutex<HashMap<String, String>>>;

#[derive(Debug, Clone)]
pub struct FakeVerifyCodeManager {
    email: Map,
    phone: Map,
}

impl FakeVerifyCodeManager {
    pub fn new() -> Self {
        Self {
            email: Arc::new(Mutex::new(HashMap::new())),
            phone: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl VerifyCodeManager for FakeVerifyCodeManager {
    async fn send_by_email(&self, email: impl Into<String>) -> Result<(), Box<dyn StdErr>> {
        let email = email.into();
        let code = Alphanumeric.sample_string(&mut thread_rng(), 6);
        println!("email: {}, code: {}", &email, &code);
        self.email.lock().unwrap().insert(email, code);
        Ok(())
    }

    async fn send_by_sms(&self, phone: impl Into<String>) -> Result<(), Box<dyn StdErr>> {
        let phone = phone.into();
        let code = Alphanumeric.sample_string(&mut thread_rng(), 6);
        println!("phone: {}, code: {}", &phone, &code);
        self.phone
            .lock()
            .map_err(|_| Error::ServiceError("锁错误".to_owned()))?
            .insert(phone, code);
        Ok(())
    }

    async fn verify_email_code(
        &mut self,
        email: impl Into<String>,
        code: impl Into<String>,
    ) -> Result<(), Box<dyn StdErr>> {
        if let Some(c) = self.email.lock().unwrap().get(&email.into()) {
            if c == &code.into() {
                return Ok(());
            }
        }
        Err(Box::new(Error::ServiceError(anyhow::anyhow!(
            "验证码不正确".to_owned()
        ))))
    }

    async fn verify_sms_code(
        &mut self,
        phone: impl Into<String>,
        code: impl Into<String>,
    ) -> Result<(), Box<dyn StdErr>> {
        for (k, v) in self.phone.lock().unwrap().iter() {
            println!("phone: {}, code: {}", k, v);
        }
        if let Some(c) = self.phone.lock().unwrap().get(&phone.into()) {
            if c == &code.into() {
                return Ok(());
            }
        }
        Err(Box::new(Error::ServiceError("验证码不正确".to_owned())))
    }
}
