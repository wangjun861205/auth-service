use crate::error::Error;
use crate::services::{self, Cacher, Hasher, Repository, SecretGenerator, VerifyCodeManager};
use actix_web::{
    web::{Json, Query},
    HttpRequest, HttpResponse,
};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListResponse<T> {
    list: Vec<T>,
    total: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterAppRequest {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterAppResponse<ID>
where
    ID: Default + Clone,
{
    id: ID,
    name: String,
    secret: String,
}

pub async fn register_app<'a, R, S, H, C, ID>(
    repository: R,
    secret_generator: S,
    hasher: H,
    cacher: C,
    Json(req): Json<RegisterAppRequest>,
) -> Result<Json<RegisterAppResponse<ID>>, Error>
where
    R: Repository<ID>,
    S: SecretGenerator,
    H: Hasher,
    C: Cacher<ID>,
    ID: Default + Clone + Serialize + Display,
{
    let res = services::register_app(
        repository,
        secret_generator,
        hasher,
        cacher,
        services::RegisterAppRequest { name: req.name },
    )
    .await?;
    Ok(Json(RegisterAppResponse {
        id: res.id,
        name: res.name,
        secret: res.secret,
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppListRequest {
    page: i32,
    size: i32,
    keywords: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct App {
    id: String,
    name: String,
    secret: String,
    secret_salt: String,
    created_at: String,
    updated_at: String,
}

pub async fn app_list<R, ID>(
    request: HttpRequest,
    mut repository: R,
    Query(req): Query<AppListRequest>,
) -> Result<Json<ListResponse<App>>, Error>
where
    R: Repository<ID>,
    ID: Default + Clone + Serialize + Display,
{
    let host = request.connection_info().host().to_owned();
    if !host.contains("localhost") {
        return Err(Error::ServiceError(
            "this endpoint is only available in local".into(),
        ));
    }
    let (apps, total) = services::app_list(
        &mut repository,
        services::AppListRequest {
            page: req.page,
            size: req.size,
            keywords: req.keywords,
        },
    )
    .await?;
    Ok(Json(ListResponse {
        list: apps
            .into_iter()
            .map(|app| App {
                id: app.id.to_string(),
                name: app.name,
                secret: app.secret,
                secret_salt: app.secret_salt,
                created_at: app.created_at.to_string(),
                updated_at: app.updated_at.to_string(),
            })
            .collect(),
        total,
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendVerifyCodeRequest {
    email: Option<String>,
    phone: Option<String>,
}

pub async fn send_verify_code<V>(
    mut verify_code_manager: V,
    Json(req): Json<SendVerifyCodeRequest>,
) -> Result<HttpResponse, Error>
where
    V: VerifyCodeManager,
{
    services::send_verify_code(
        &mut verify_code_manager,
        services::SendVerifyCodeRequest {
            email: req.email,
            phone: req.phone,
        },
    )
    .await?;
    Ok(HttpResponse::Ok().finish())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterUserRequest<ID> {
    phone: Option<String>,
    email: Option<String>,
    password: String,
    verify_code: String,
    app_id: ID,
    app_secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterUserResponse<ID> {
    id: ID,
    secret: String,
}

pub async fn register_user<R, S, V, H, C, ID>(
    repository: R,
    secret_generator: S,
    verify_code_manager: V,
    hasher: H,
    cacher: C,
    Json(req): Json<RegisterUserRequest<ID>>,
) -> Result<Json<RegisterUserResponse<ID>>, Error>
where
    R: Repository<ID>,
    S: SecretGenerator,
    V: VerifyCodeManager,
    H: Hasher,
    C: Cacher<ID>,
    ID: Default + Clone + Serialize + for<'de> Deserialize<'de> + Display,
{
    let services::RegisterUserResponse { id, secret } = services::register_user(
        repository,
        secret_generator,
        verify_code_manager,
        hasher,
        cacher,
        services::RegisterUserRequest {
            phone: req.phone,
            email: req.email,
            password: req.password,
            verify_code: req.verify_code,
            app_id: req.app_id,
            app_secret: req.app_secret,
        },
    )
    .await?;
    Ok(Json(RegisterUserResponse { id, secret }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest<ID> {
    phone: Option<String>,
    email: Option<String>,
    password: String,
    app_id: ID,
    app_secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse<ID> {
    id: ID,
    secret: String,
}

pub async fn login<R, S, H, C, ID>(
    repository: R,
    secret_generator: S,
    hasher: H,
    cacher: C,
    Json(LoginRequest {
        phone,
        email,
        password,
        app_id,
        app_secret,
    }): Json<LoginRequest<ID>>,
) -> Result<Json<LoginResponse<ID>>, Error>
where
    R: Repository<ID>,
    S: SecretGenerator,
    H: Hasher,
    C: Cacher<ID>,
    ID: Default + Clone + Serialize + for<'de> Deserialize<'de> + Display,
{
    let services::LoginResponse { id, secret } = services::login(
        repository,
        secret_generator,
        hasher,
        cacher,
        services::LoginRequest {
            phone,
            email,
            password,
            app_id,
            app_secret,
        },
    )
    .await?;
    Ok(Json(LoginResponse { id, secret }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifySecretRequest<ID> {
    id: ID,
    secret: String,
    app_id: ID,
    app_secret: String,
}

pub async fn verify_secret<R, H, C, ID>(
    repository: R,
    hasher: H,
    cacher: C,
    Json(VerifySecretRequest {
        id,
        secret,
        app_id,
        app_secret,
    }): Json<VerifySecretRequest<ID>>,
) -> Result<HttpResponse, Error>
where
    R: Repository<ID>,
    H: Hasher,
    C: Cacher<ID>,
    ID: Default + Clone + Serialize + for<'de> Deserialize<'de> + Display,
{
    services::verify_secret(
        repository,
        hasher,
        cacher,
        services::VerifySecretRequest {
            id,
            secret,
            app_id,
            app_secret,
        },
    )
    .await?;
    Ok(HttpResponse::Ok().finish())
}
