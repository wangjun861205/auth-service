use std::error::Error as StdErr;

pub trait Hasher {
    fn generate_salt(&self) -> Result<String, Box<dyn StdErr>>;
    fn hash(
        &self,
        content: impl Into<String>,
        salt: impl Into<String>,
    ) -> Result<String, Box<dyn StdErr>>;
}
