use crate::core::secret_generator::SecretGenerator;
use std::error::Error as StdErr;
use uuid::Uuid;

#[derive(Debug, Default, Clone)]
pub struct RandomSecretGenerator;

impl SecretGenerator for RandomSecretGenerator {
    fn generate_secret(&self) -> Result<String, Box<dyn StdErr>> {
        Ok(Uuid::new_v4().to_string())
    }
}
