use std::error::Error as StdErr;
pub trait VerifyCodeManager {
    async fn send_by_sms(&self, phone: impl Into<String>) -> Result<(), Box<dyn StdErr>>;

    async fn send_by_email(&self, email: impl Into<String>) -> Result<(), Box<dyn StdErr>>;

    async fn verify_sms_code(
        &mut self,
        phone: impl Into<String>,
        code: impl Into<String>,
    ) -> Result<(), Box<dyn StdErr>>;
    async fn verify_email_code(
        &mut self,
        email: impl Into<String>,
        code: impl Into<String>,
    ) -> Result<(), Box<dyn StdErr>>;
}
