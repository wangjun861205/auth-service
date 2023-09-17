use std::{
    error::Error as StdErr,
    fmt::{Debug, Display},
};

#[derive(Debug)]
pub enum Error<E>
where
    E: Debug + Display,
{
    CacherError(E),
    HasherError(E),
    RepositoryError(E),
    SecretGeneratorError(E),
    VerifyCodeManagerError(E),
    ServiceError(E),
}

impl<E> Display for Error<E>
where
    E: Debug + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::CacherError(e) => write!(f, "CacherError: {}", e),
            Error::HasherError(e) => write!(f, "HasherError: {}", e),
            Error::RepositoryError(e) => write!(f, "RepositoryError: {}", e),
            Error::SecretGeneratorError(e) => write!(f, "SecretGeneratorError: {}", e),
            Error::VerifyCodeManagerError(e) => write!(f, "VerifyCodeManagerError: {}", e),
            Error::ServiceError(e) => write!(f, "ServiceError: {}", e),
        }
    }
}

impl<E> StdErr for Error<E> where E: Debug + Display {}
