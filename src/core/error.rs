use std::fmt::{Debug, Display};

#[derive(Debug)]
pub enum Error {
    RepositoryError(Box<dyn Debug>),
    HasherError(Box<dyn Debug>),
    TokenManagerError(Box<dyn Debug>),
    FailedToSignToken(Box<dyn Debug>),
    FailedToVerifyToken(Box<dyn Debug>),
    FailedToCheckExists(Box<dyn Debug>),
    FailedToFetchUser(Box<dyn Debug>),
    FailedToInsertUser(Box<dyn Debug>),
    FailedToGenerateSalt(Box<dyn Debug>),
    IdentifierAlreadyExists,
    UserNotExists,
    InvalidCredential,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::RepositoryError(e) => write!(f, "Repository error: {:?}", e),
            Error::HasherError(e) => write!(f, "Hasher error: {:?}", e),
            Error::TokenManagerError(e) => write!(f, "Token manager error: {:?}", e),
            Error::FailedToSignToken(e) => write!(f, "Failed to sign token: {:?}", e),
            Error::FailedToVerifyToken(e) => write!(f, "Failed to verify token: {:?}", e),
            Error::FailedToCheckExists(e) => write!(f, "Failed to check exists: {:?}", e),
            Error::FailedToFetchUser(e) => write!(f, "Failed to fetch user: {:?}", e),
            Error::FailedToInsertUser(e) => write!(f, "Failed to insert user: {:?}", e),
            Error::FailedToGenerateSalt(e) => write!(f, "Failed to generate salt: {:?}", e),
            Error::IdentifierAlreadyExists => write!(f, "Identifier already exists"),
            Error::UserNotExists => write!(f, "User not exists"),
            Error::InvalidCredential => write!(f, "Invalid credential"),
        }
    }
}
