use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::models::{App, CreateApp, CreateUser, QueryApp, QueryUser, UpdateUser, User};
use std::fmt::Display;
pub trait Repository<ID>
where
    ID: Default + Clone + Display,
{
    async fn insert_app(&mut self, app: CreateApp) -> Result<ID, Error>;
    async fn fetch_app(&mut self, query: QueryApp<ID>) -> Result<Option<App<ID>>, Error>;
    async fn exists_app(&mut self, query: QueryApp<ID>) -> Result<bool, Error>;
    async fn query_apps(
        &mut self,
        query: QueryApp<ID>,
        page: i32,
        size: i32,
    ) -> Result<Vec<App<ID>>, Error>;
    async fn delete_app(&mut self, id: ID) -> Result<i64, Error>;
    async fn count_apps(&mut self, query: QueryApp<ID>) -> Result<i64, Error>;
    async fn insert_user(&mut self, user: CreateUser<ID>) -> Result<ID, Error>;
    async fn fetch_user(&mut self, query: QueryUser<ID>) -> Result<Option<User<ID>>, Error>;
    async fn update_user(&mut self, query: QueryUser<ID>, user: UpdateUser) -> Result<i64, Error>;
    async fn commit(self) -> Result<(), Error>;
}

pub trait VerifyCodeManager {
    async fn send_by_sms(&mut self, phone: impl Into<String>) -> Result<(), Error>;

    async fn send_by_email(&mut self, email: impl Into<String>) -> Result<(), Error>;
    async fn verify_sms_code(
        &mut self,
        phone: impl Into<String>,
        code: impl Into<String>,
    ) -> Result<(), Error>;
    async fn verify_email_code(
        &mut self,
        email: impl Into<String>,
        code: impl Into<String>,
    ) -> Result<(), Error>;
}

pub trait SecretGenerator {
    fn generate_secret(&self) -> Result<String, Error>;
}

pub trait Hasher {
    fn generate_salt(&self) -> Result<String, Error>;
    fn hash(&self, content: impl Into<String>, salt: impl Into<String>) -> Result<String, Error>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SecretPair {
    pub hashed_secret: String,
    pub secret_salt: String,
}

pub trait Cacher<ID> {
    async fn put_app_secret(&mut self, id: ID, pair: SecretPair) -> Result<(), Error>;
    async fn get_app_secret(&mut self, id: ID) -> Result<Option<SecretPair>, Error>;
    async fn delete_app_secret(&mut self, id: ID) -> Result<(), Error>;
    async fn put_user_secret(&mut self, id: ID, secret: SecretPair) -> Result<(), Error>;
    async fn get_user_secret(&mut self, id: ID) -> Result<Option<SecretPair>, Error>;
}

//===============================以下为服务==================================

pub struct RegisterAppRequest {
    pub name: String,
}

pub struct RegisterAppResponse<ID> {
    pub id: ID,
    pub name: String,
    pub secret: String,
}

pub async fn register_app<R, S, H, C, ID>(
    mut repository: R,
    secret_generator: S,
    hasher: H,
    mut cacher: C,
    req: RegisterAppRequest,
) -> Result<RegisterAppResponse<ID>, Error>
where
    R: Repository<ID>,
    S: SecretGenerator,
    H: Hasher,
    C: Cacher<ID>,
    ID: Default + Clone + Display,
{
    let secret = secret_generator.generate_secret()?;
    let secret_salt = hasher.generate_salt()?;
    let hashed_secret = hasher.hash(&secret, &secret_salt)?;
    let id = repository
        .insert_app(CreateApp {
            name: req.name.clone(),
            secret: hashed_secret.clone(),
            secret_salt: secret_salt.clone(),
        })
        .await?;
    repository.commit().await?;
    if let Err(err) = cacher
        .put_app_secret(
            id.clone(),
            SecretPair {
                hashed_secret,
                secret_salt,
            },
        )
        .await
    {
        eprintln!("{:?}", err);
    }
    Ok(RegisterAppResponse {
        id,
        name: req.name,
        secret,
    })
}

pub struct AppListRequest {
    pub page: i32,
    pub size: i32,
    pub keywords: Option<String>,
}

pub async fn app_list<R, ID>(
    repository: &mut R,
    req: AppListRequest,
) -> Result<(Vec<App<ID>>, i64), Error>
where
    R: Repository<ID>,
    ID: Default + Clone + Display,
{
    let query = QueryApp {
        name_like_any: req
            .keywords
            .map(|keywords| keywords.split_whitespace().map(str::to_owned).collect()),
        ..Default::default()
    };
    let total = repository.count_apps(query.clone()).await?;
    let apps = repository.query_apps(query, req.page, req.size).await?;
    Ok((apps, total))
}

pub async fn delete_app<R, C, ID>(repository: &mut R, mut cacher: C, id: ID) -> Result<i64, Error> {
    let deleted = repository.delete_app(id.clone()).await?;
    cacher.delete_app_secret(id).await?;
    Ok(deleted)
}

async fn verify_app_secret<R, H, C, ID>(
    repository: &mut R,
    hasher: &H,
    cacher: &mut C,
    id: ID,
    secret: impl Into<String>,
) -> Result<(), Error>
where
    R: Repository<ID>,
    H: Hasher,
    C: Cacher<ID>,
    ID: Default + Clone + Display,
{
    let secret = secret.into();
    if let Some(SecretPair {
        hashed_secret,
        secret_salt,
    }) = cacher
        .get_app_secret(id.clone())
        .await
        .unwrap_or_else(|err| {
            eprintln!("{:?}", err);
            None
        })
    {
        if hasher.hash(secret.clone(), secret_salt)? == hashed_secret {
            return Ok(());
        }
        return Err(Error::ServiceError("应用凭证不正确".into()));
    }
    if let Some(app) = repository
        .fetch_app(QueryApp {
            id_eq: Some(id.clone()),
            ..Default::default()
        })
        .await?
    {
        let hashed_secret = hasher.hash(secret, &app.secret_salt)?;
        if hashed_secret != app.secret {
            return Err(Error::ServiceError("应用凭证不正确".into()));
        }
        cacher
            .put_app_secret(
                id,
                SecretPair {
                    hashed_secret,
                    secret_salt: app.secret_salt,
                },
            )
            .await
            .unwrap_or_else(|e| {
                eprintln!("{:?}", e);
            });
        return Ok(());
    }
    Err(Error::ServiceError("应用凭证不正确".into()))
}

pub struct SendVerifyCodeRequest {
    pub phone: Option<String>,
    pub email: Option<String>,
}

pub async fn send_verify_code<V>(
    verify_code_manager: &mut V,
    req: SendVerifyCodeRequest,
) -> Result<(), Error>
where
    V: VerifyCodeManager,
{
    if req.phone.is_none() && req.email.is_none() {
        return Err(Error::ServiceError("手机号与邮箱至少提供一个".into()));
    }
    if let Some(phone) = req.phone {
        verify_code_manager.send_by_sms(phone).await?;
    }
    if let Some(email) = req.email {
        verify_code_manager.send_by_email(email).await?;
    }
    Ok(())
}

pub struct RegisterUserRequest<ID> {
    pub phone: Option<String>,
    pub email: Option<String>,
    pub password: String,
    pub verify_code: String,
    pub app_id: ID,
    pub app_secret: String,
}

pub struct RegisterUserResponse<ID> {
    pub id: ID,
    pub secret: String,
}

pub async fn register_user<R, S, H, V, C, ID>(
    mut repository: R,
    secret_generator: S,
    mut verify_code_manager: V,
    hasher: H,
    mut cacher: C,
    req: RegisterUserRequest<ID>,
) -> Result<RegisterUserResponse<ID>, Error>
where
    R: Repository<ID>,
    S: SecretGenerator,
    V: VerifyCodeManager,
    H: Hasher,
    C: Cacher<ID>,
    ID: Default + Clone + Display,
{
    if req.phone.is_none() && req.email.is_none() {
        return Err(Error::ServiceError("手机号与邮箱至少提供一个".into()));
    }
    verify_app_secret(
        &mut repository,
        &hasher,
        &mut cacher,
        req.app_id.clone(),
        req.app_secret,
    )
    .await?;
    if let Some(phone) = &req.phone {
        verify_code_manager
            .verify_sms_code(phone, &req.verify_code)
            .await?;
    }
    if let Some(email) = &req.email {
        verify_code_manager
            .verify_email_code(email, &req.verify_code)
            .await?;
    }

    let secret = secret_generator.generate_secret()?;
    let secret_salt = hasher.generate_salt()?;
    let hashed_secret = hasher.hash(&secret, &secret_salt)?;
    let password_salt = hasher.generate_salt()?;
    let hashed_password = hasher.hash(&req.password, &password_salt)?;
    let id = repository
        .insert_user(CreateUser {
            phone: req.phone,
            email: req.email,
            password: Some(hashed_password),
            password_salt: Some(password_salt),
            secret: hashed_secret.clone(),
            secret_salt: secret_salt.clone(),
            app_id: req.app_id,
        })
        .await?;
    repository.commit().await?;
    cacher
        .put_user_secret(
            id.clone(),
            SecretPair {
                hashed_secret,
                secret_salt,
            },
        )
        .await?;
    Ok(RegisterUserResponse { id, secret })
}

pub struct VerifySecretRequest<ID> {
    pub id: ID,
    pub secret: String,
    pub app_id: ID,
    pub app_secret: String,
}

pub async fn verify_secret<R, H, C, ID>(
    mut repository: R,
    hasher: H,
    mut cacher: C,
    req: VerifySecretRequest<ID>,
) -> Result<(), Error>
where
    R: Repository<ID>,
    H: Hasher,
    C: Cacher<ID>,
    ID: Default + Clone + Display,
{
    verify_app_secret(
        &mut repository,
        &hasher,
        &mut cacher,
        req.app_id.clone(),
        req.app_secret,
    )
    .await?;
    if let Some(SecretPair {
        hashed_secret,
        secret_salt,
    }) = cacher
        .get_user_secret(req.id.clone())
        .await
        .unwrap_or_else(|err| {
            eprintln!("{:?}", err);
            None
        })
    {
        if hasher.hash(&req.secret, secret_salt)? == hashed_secret {
            return Ok(());
        }
        return Err(Error::ServiceError("用户不存在或凭证不正确".into()));
    }
    if let Some(user) = repository
        .fetch_user(QueryUser {
            id_eq: Some(req.id),
            app_id_eq: Some(req.app_id),
            ..Default::default()
        })
        .await?
    {
        if hasher.hash(&req.secret, &user.secret_salt)? != user.secret {
            return Err(Error::ServiceError("用户不存在或凭证不正确".into()));
        }
        return Ok(());
    }
    Err(Error::ServiceError("用户不存在或凭证不正确".into()))
}

pub struct LoginRequest<ID> {
    pub phone: Option<String>,
    pub email: Option<String>,
    pub password: String,
    pub app_id: ID,
    pub app_secret: String,
}

pub struct LoginResponse<ID> {
    pub id: ID,
    pub secret: String,
}

pub async fn login<R, S, H, C, ID>(
    mut repository: R,
    secret_generator: S,
    hasher: H,
    mut cacher: C,
    req: LoginRequest<ID>,
) -> Result<LoginResponse<ID>, Error>
where
    R: Repository<ID>,
    S: SecretGenerator,
    H: Hasher,
    C: Cacher<ID>,
    ID: Clone + Default + Display,
{
    if req.phone.is_none() && req.email.is_none() {
        return Err(Error::ServiceError("手机号与邮箱至少提供一个".into()));
    }
    verify_app_secret(
        &mut repository,
        &hasher,
        &mut cacher,
        req.app_id.clone(),
        req.app_secret,
    )
    .await?;
    if let Some(user) = repository
        .fetch_user(QueryUser {
            phone_eq: req.phone,
            email_eq: req.email,
            app_id_eq: Some(req.app_id),
            ..Default::default()
        })
        .await?
    {
        if user.password.is_none() || user.password_salt.is_none() {
            return Err(Error::ServiceError("不支持的登录方式".into()));
        }
        if hasher.hash(&req.password, user.password_salt.unwrap())? != user.password.unwrap() {
            return Err(Error::ServiceError("用户不存在或凭证不正确".into()));
        }
        let secret = secret_generator.generate_secret()?;
        let salt = hasher.generate_salt()?;
        let hashed_secret = hasher.hash(&secret, &salt)?;
        let affected = repository
            .update_user(
                QueryUser {
                    id_eq: Some(user.id.clone()),
                    ..Default::default()
                },
                UpdateUser {
                    secret: Some(hashed_secret.clone()),
                    secret_salt: Some(salt.clone()),
                },
            )
            .await?;
        if affected != 1 {
            return Err(Error::ServiceError("更新用户密钥失败".into()));
        }
        repository.commit().await?;
        cacher
            .put_user_secret(
                user.id.clone(),
                SecretPair {
                    hashed_secret,
                    secret_salt: salt,
                },
            )
            .await?;
        return Ok(LoginResponse {
            id: user.id,
            secret,
        });
    }
    Err(Error::ServiceError("用户不存在或凭证不正确".into()))
}
