use std::fmt::Display;

use actix_web::{HttpResponse, ResponseError};

#[derive(Debug)]
pub enum Error {
    RepositoryError(String),
    ServiceError(String),
}

impl ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Self::RepositoryError(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            Self::ServiceError(_) => actix_web::http::StatusCode::BAD_REQUEST,
        }
    }
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        match self {
            Self::RepositoryError(msg) => {
                HttpResponse::InternalServerError().body(format!("存储错误: {}", msg))
            }
            Self::ServiceError(msg) => HttpResponse::InternalServerError().body(format!("{}", msg)),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        Self::RepositoryError(format!("存储错误: {}", value))
    }
}
