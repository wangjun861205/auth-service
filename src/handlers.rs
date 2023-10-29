use crate::core::{
    cacher::Cacher,
    error::Error,
    hasher::Hasher,
    repository::Repository,
    service::{self, Service},
    token_manager::SecretGenerator,
};
use actix_web::{
    web::{Data, Json},
    HttpRequest, HttpResponse,
};
use serde::{Deserialize, Serialize};
use std::{
    error::Error as StdErr,
    fmt::{Debug, Display},
    str::FromStr,
};
use tokio::sync::Mutex;

pub type SyncService<R, C, H, S, ID> = Data<Mutex<Service<R, C, H, S, ID>>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListResponse<T> {
    list: Vec<T>,
    total: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteResponse {
    deleted: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendVerifyCodeRequest {
    email: Option<String>,
    phone: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterUserRequest {
    phone: Option<String>,
    email: Option<String>,
    password: String,
    verify_code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterUserResponse<ID> {
    id: ID,
    secret: String,
}

pub async fn register_user<R, S, H, C, ID>(
    service: SyncService<R, C, H, S, ID>,
    Json(req): Json<RegisterUserRequest>,
) -> Result<Json<RegisterUserResponse<ID>>, Box<dyn StdErr>>
where
    R: Repository<ID> + Clone,
    S: SecretGenerator + Clone,
    H: Hasher + Clone,
    C: Cacher<ID> + Clone,
    ID: Default + Clone + Serialize + for<'de> Deserialize<'de> + Display,
{
    let service::RegisterUserResponse { id, secret } = service
        .lock()
        .await
        .register_user(service::RegisterUserRequest {
            phone: req.phone,
            email: req.email,
            password: req.password,
            verify_code: req.verify_code,
        })
        .await?;
    Ok(Json(RegisterUserResponse { id, secret }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    phone: Option<String>,
    email: Option<String>,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse<ID> {
    id: ID,
    secret: String,
}

pub async fn login<R, S, H, C, ID>(
    service: SyncService<R, C, H, S, ID>,
    Json(LoginRequest {
        phone,
        email,
        password,
    }): Json<LoginRequest>,
) -> Result<Json<LoginResponse<ID>>, Box<dyn StdErr>>
where
    R: Repository<ID> + Clone,
    S: SecretGenerator + Clone,
    H: Hasher + Clone,
    C: Cacher<ID> + Clone,
    ID: Default + Clone + Serialize + for<'de> Deserialize<'de> + Display,
{
    let service::LoginResponse { id, secret } = service
        .lock()
        .await
        .login(service::LoginRequest {
            phone,
            email,
            password,
        })
        .await?;
    Ok(Json(LoginResponse { id, secret }))
}

pub struct UIDHeader(pub String);
pub struct SecretHeader(pub String);

pub async fn verify_secret<R, H, C, S, ID, FE>(
    req: HttpRequest,
    uid_header: Data<UIDHeader>,
    secret_header: Data<SecretHeader>,
    service: SyncService<R, C, H, S, ID>,
) -> Result<HttpResponse, Box<dyn StdErr>>
where
    R: Repository<ID> + Clone,
    H: Hasher + Clone,
    C: Cacher<ID> + Clone,
    S: SecretGenerator + Clone,
    ID: Default + Clone + Serialize + for<'de> Deserialize<'de> + Display + FromStr<Err = FE>,
    FE: Debug + Display,
{
    let id = req
        .headers()
        .get(&uid_header.0)
        .ok_or(Box::new(Error::ServiceError(
            "uid header not found".to_string(),
        )))?
        .to_str()?
        .parse::<ID>()
        .map_err(|e| Box::new(Error::ServiceError(e.to_string())))?;
    let secret = req
        .headers()
        .get(&secret_header.0)
        .ok_or(Box::new(Error::ServiceError(
            "token header not found".to_string(),
        )))?
        .to_str()?
        .parse::<String>()
        .map_err(|e| Box::new(Error::ServiceError(e.to_string())))?;
    service
        .lock()
        .await
        .verify_secret(service::VerifySecretRequest { id, secret })
        .await?;
    Ok(HttpResponse::Ok().finish())
}
