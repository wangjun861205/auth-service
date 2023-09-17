use std::error::Error as StdErr;
pub trait SecretGenerator {
    fn generate_secret(&self) -> Result<String, Box<dyn StdErr>>;
}
