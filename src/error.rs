use std::fmt::Display;

use actix_web::{HttpResponse, ResponseError};
use redis::RedisError;

#[derive(Debug)]
pub enum Error {
    RepositoryError(String),
    ServiceError(String),
    ServerError(String),
    CacherError(String),
}

impl ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Self::RepositoryError(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            Self::ServiceError(_) => actix_web::http::StatusCode::BAD_REQUEST,
            Self::ServerError(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            Self::CacherError(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        match self {
            Self::RepositoryError(msg) => {
                HttpResponse::InternalServerError().body(format!("存储错误: {}", msg))
            }
            Self::ServiceError(msg) => HttpResponse::InternalServerError().body(format!("{}", msg)),
            Self::ServerError(msg) => {
                HttpResponse::InternalServerError().body(format!("服务器错误: {}", msg))
            }
            Self::CacherError(msg) => {
                HttpResponse::InternalServerError().body(format!("缓存错误: {}", msg))
            }
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

impl From<RedisError> for Error {
    fn from(value: RedisError) -> Self {
        Self::CacherError(format!("缓存错误: {}", value))
    }
}
