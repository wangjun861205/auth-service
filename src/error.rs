use std::fmt::Display;

use actix_web::{HttpResponse, ResponseError};
use redis::RedisError;

#[derive(Debug)]
pub enum Error {
    RepositoryError(String),
    ServiceError(String),
    ServerError(String),
    CacherError(String),
    SerdeError(String),
}

impl ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Self::RepositoryError(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            Self::ServiceError(_) => actix_web::http::StatusCode::BAD_REQUEST,
            Self::ServerError(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            Self::CacherError(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            Self::SerdeError(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        match self {
            Self::RepositoryError(msg) => {
                HttpResponse::InternalServerError().body(format!("存储错误: {}", msg))
            }
            Self::ServiceError(msg) => {
                HttpResponse::InternalServerError().body(format!("服务错误: {}", msg))
            }
            Self::ServerError(msg) => {
                HttpResponse::InternalServerError().body(format!("服务器错误: {}", msg))
            }
            Self::CacherError(msg) => {
                HttpResponse::InternalServerError().body(format!("缓存错误: {}", msg))
            }
            Self::SerdeError(msg) => {
                HttpResponse::InternalServerError().body(format!("序列化错误: {}", msg.clone()))
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
        Self::RepositoryError(value.to_string())
    }
}

impl From<RedisError> for Error {
    fn from(value: RedisError) -> Self {
        Self::CacherError(value.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::SerdeError(value.to_string())
    }
}
