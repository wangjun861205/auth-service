use std::f32::consts::E;
use std::hash;

use crate::error::Error;
use crate::models::{App, CreateApp, CreateUser, QueryApp, QueryUser, UpdateUser, User};
pub trait Repository<ID>
where
    ID: Default + Clone,
{
    async fn insert_app(&mut self, app: CreateApp) -> Result<ID, Error>;
    async fn fetch_app(&mut self, query: QueryApp<ID>) -> Result<Option<App<ID>>, Error>;
    async fn insert_user(&mut self, user: CreateUser<ID>) -> Result<ID, Error>;
    async fn fetch_user(&mut self, query: QueryUser<ID>) -> Result<Option<User<ID>>, Error>;
    async fn update_user(&mut self, query: QueryUser<ID>, user: UpdateUser) -> Result<i64, Error>;
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

pub trait EmailSender {
    async fn send(
        &mut self,
        email: impl AsRef<String>,
        content: impl AsRef<String>,
    ) -> Result<(), Error>;
}

pub trait SecretGenerator {
    fn generate_secret(&self) -> Result<String, Error>;
    fn generate_salt(&self) -> Result<String, Error>;
    fn hash(&self, content: impl Into<String>, salt: impl Into<String>) -> Result<String, Error>;
}

//===============================以下为服务==================================

pub struct RegisterAppRequest {
    pub name: String,
}

pub struct RegisterAppResponse<ID> {
    pub id: ID,
    pub secret: String,
}

pub async fn register_app<R, S, ID>(
    repository: &mut R,
    secret_generator: &S,
    req: RegisterAppRequest,
) -> Result<RegisterAppResponse<ID>, Error>
where
    R: Repository<ID>,
    S: SecretGenerator,
    ID: Default + Clone,
{
    let secret = secret_generator.generate_secret()?;
    let secret_salt = secret_generator.generate_salt()?;
    let hashed_secret = secret_generator.hash(&secret, &secret_salt)?;
    let id = repository
        .insert_app(CreateApp {
            name: req.name,
            secret: hashed_secret,
            secret_salt,
        })
        .await?;
    Ok(RegisterAppResponse { id, secret })
}

async fn verify_app_secret<R, S, ID>(
    repository: &mut R,
    secret_generator: &S,
    id: ID,
    secret: impl Into<String>,
) -> Result<(), Error>
where
    R: Repository<ID>,
    S: SecretGenerator,
    ID: Default + Clone,
{
    if let Some(app) = repository.fetch_app(QueryApp { id_eq: Some(id) }).await? {
        let hashed_secret = secret_generator.hash(secret, &app.secret_salt)?;
        if hashed_secret != app.secret {
            return Err(Error::ServiceError("应用凭证不正确".into()));
        }
        return Ok(());
    }
    return Err(Error::ServiceError("应用凭证不正确".into()));
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

pub struct RegisterUserWithVerifyCodeRequest<ID> {
    phone: Option<String>,
    email: Option<String>,
    verify_code: String,
    app_id: ID,
    app_secret: String,
}

pub struct RegisterUserWithVerifyCodeResponse<ID> {
    id: ID,
    secret: String,
}

pub async fn register_user_with_verify_code<R, S, V, ID>(
    repository: &mut R,
    secret_generator: &S,
    verify_code_manager: &mut V,
    req: RegisterUserWithVerifyCodeRequest<ID>,
) -> Result<RegisterUserWithVerifyCodeResponse<ID>, Error>
where
    R: Repository<ID>,
    S: SecretGenerator,
    V: VerifyCodeManager,
    ID: Default + Clone,
{
    verify_app_secret(
        repository,
        secret_generator,
        req.app_id.clone(),
        req.app_secret,
    )
    .await?;
    if req.phone.is_none() && req.email.is_none() {
        return Err(Error::ServiceError("手机号与邮箱至少提供一个".into()));
    }
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
    let secret_salt = secret_generator.generate_salt()?;
    let hashed_secret = secret_generator.hash(&secret, &secret_salt)?;
    let id = repository
        .insert_user(CreateUser {
            phone: req.phone,
            email: req.email,
            secret: hashed_secret,
            secret_salt,
            app_id: req.app_id,
            ..Default::default()
        })
        .await?;
    Ok(RegisterUserWithVerifyCodeResponse { id, secret })
}

pub struct RegisterUserWithPasswordRequest<ID> {
    pub phone: Option<String>,
    pub email: Option<String>,
    pub password: String,
    pub app_id: ID,
    pub app_secret: String,
}

pub struct RegisterUserWithPasswordResponse<ID> {
    id: ID,
    secret: String,
}

pub async fn register_user_with_password<R, S, V, ID>(
    repository: &mut R,
    secret_generator: &S,
    req: RegisterUserWithPasswordRequest<ID>,
) -> Result<RegisterUserWithPasswordResponse<ID>, Error>
where
    R: Repository<ID>,
    S: SecretGenerator,
    V: VerifyCodeManager,
    ID: Default + Clone,
{
    verify_app_secret(
        repository,
        secret_generator,
        req.app_id.clone(),
        req.app_secret,
    )
    .await?;
    if req.phone.is_none() && req.email.is_none() {
        return Err(Error::ServiceError("手机号与邮箱至少提供一个".into()));
    }
    let secret = secret_generator.generate_secret()?;
    let secret_salt = secret_generator.generate_salt()?;
    let hashed_secret = secret_generator.hash(&secret, &secret_salt)?;
    let password_salt = secret_generator.generate_salt()?;
    let hashed_password = secret_generator.hash(&req.password, &password_salt)?;
    let id = repository
        .insert_user(CreateUser {
            phone: req.phone,
            email: req.email,
            password: Some(hashed_password),
            password_salt: Some(password_salt),
            secret: hashed_secret,
            secret_salt,
            app_id: req.app_id,
        })
        .await?;
    Ok(RegisterUserWithPasswordResponse { id, secret })
}

pub struct LoginBySecretRequest<ID> {
    id: ID,
    secret: String,
    app_id: ID,
    app_secret: String,
}

pub async fn login_by_secret<R, S, ID>(
    repository: &mut R,
    secret_generator: &S,
    req: LoginBySecretRequest<ID>,
) -> Result<(), Error>
where
    R: Repository<ID>,
    S: SecretGenerator,
    ID: Clone + Default,
{
    verify_app_secret(
        repository,
        secret_generator,
        req.app_id.clone(),
        req.app_secret,
    )
    .await?;
    if let Some(user) = repository
        .fetch_user(QueryUser {
            id_eq: Some(req.id),
            app_id_eq: Some(req.app_id),
            ..Default::default()
        })
        .await?
    {
        if secret_generator.hash(&req.secret, &user.secret_salt)? != user.secret {
            return Err(Error::ServiceError("用户不存在或凭证不正确".into()));
        }
        return Ok(());
    }
    Err(Error::ServiceError("用户不存在或凭证不正确".into()))
}

pub struct LoginByPasswordRequest<ID> {
    phone: Option<String>,
    email: Option<String>,
    password: String,
    app_id: ID,
    app_secret: String,
}

pub async fn login_by_password<R, S, ID>(
    repository: &mut R,
    secret_generator: &S,
    req: LoginByPasswordRequest<ID>,
) -> Result<(), Error>
where
    R: Repository<ID>,
    S: SecretGenerator,
    ID: Clone + Default,
{
    if req.phone.is_none() && req.email.is_none() {
        return Err(Error::ServiceError("手机号与邮箱至少提供一个".into()));
    }
    verify_app_secret(
        repository,
        secret_generator,
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
        if secret_generator.hash(&req.password, user.password_salt.unwrap())?
            != user.password.unwrap()
        {
            return Err(Error::ServiceError("用户不存在或凭证不正确".into()));
        }
        return Ok(());
    }
    Err(Error::ServiceError("用户不存在或凭证不正确".into()))
}

pub struct RefreshSecretByVerifyCodeRequest {
    phone: Option<String>,
    email: Option<String>,
    verify_code: String,
}

pub struct RefreshSecretByVerifyCodeResponse<ID> {
    id: ID,
    secret: String,
}

pub async fn refresh_secret_by_verify_code<R, S, V, ID>(
    repository: &mut R,
    secret_generator: &S,
    verify_code_manager: &mut V,
    req: RefreshSecretByVerifyCodeRequest,
) -> Result<RefreshSecretByVerifyCodeResponse<ID>, Error>
where
    R: Repository<ID>,
    S: SecretGenerator,
    V: VerifyCodeManager,
    ID: Default + Clone,
{
    if req.phone.is_none() && req.email.is_none() {
        return Err(Error::ServiceError("手机号与邮箱至少提供一个".into()));
    }
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
    if let Some(user) = repository
        .fetch_user(QueryUser {
            phone_eq: req.phone,
            email_eq: req.email,
            ..Default::default()
        })
        .await?
    {
        let secret = secret_generator.generate_secret()?;
        let secret_salt = secret_generator.generate_salt()?;
        let hashed_secret = secret_generator.hash(&secret, &secret_salt)?;
        repository
            .update_user(
                QueryUser {
                    id_eq: Some(user.id.clone()),
                    ..Default::default()
                },
                UpdateUser {
                    secret: Some(hashed_secret),
                    secret_salt: Some(secret_salt),
                },
            )
            .await?;
        return Ok(RefreshSecretByVerifyCodeResponse {
            id: user.id,
            secret,
        });
    }
    Err(Error::ServiceError("用户不存在或验证码不正确".into()))
}
