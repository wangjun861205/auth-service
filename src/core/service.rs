use crate::core::{
    cacher::{Cacher, SecretPair},
    entities::{CreateUser, QueryUser, UpdateUser},
    error::Error,
    hasher::Hasher,
    repository::Repository,
    secret_generator::SecretGenerator,
};

use std::{error::Error as StdErr, fmt::Display, marker::PhantomData};

#[derive(Debug, Clone)]
pub struct Service<R, C, H, S, ID>
where
    R: Repository<ID> + Clone,
    C: Cacher<ID> + Clone,
    H: Hasher + Clone,
    S: SecretGenerator + Clone,
    ID: Default + Clone + Display,
{
    pub repository: R,
    pub cacher: C,
    pub hasher: H,
    pub secret_generator: S,
    _phantom: PhantomData<ID>,
}

pub struct SendVerifyCodeRequest {
    pub phone: Option<String>,
    pub email: Option<String>,
}

pub struct RegisterUserRequest {
    pub phone: Option<String>,
    pub email: Option<String>,
    pub password: String,
    pub verify_code: String,
}

pub struct RegisterUserResponse<ID> {
    pub id: ID,
    pub secret: String,
}

pub struct VerifySecretRequest<ID> {
    pub id: ID,
    pub secret: String,
}

pub struct LoginRequest {
    pub phone: Option<String>,
    pub email: Option<String>,
    pub password: String,
}

pub struct LoginResponse<ID> {
    pub id: ID,
    pub secret: String,
}

impl<R, C, H, S, ID> Service<R, C, H, S, ID>
where
    R: Repository<ID> + Clone,
    C: Cacher<ID> + Clone,
    H: Hasher + Clone,
    S: SecretGenerator + Clone,
    ID: Default + Clone + Display,
{
    pub fn new(repository: R, cacher: C, hasher: H, secret_generator: S) -> Self {
        Self {
            repository,
            cacher,
            hasher,
            secret_generator,
            _phantom: PhantomData,
        }
    }

    pub async fn register_user(
        &mut self,
        req: RegisterUserRequest,
    ) -> Result<RegisterUserResponse<ID>, Box<dyn StdErr>> {
        if req.phone.is_none() && req.email.is_none() {
            return Err(Box::new(Error::ServiceError(
                "手机号与邮箱至少提供一个".to_owned(),
            )));
        }
        let secret = self.secret_generator.generate_secret()?;
        let secret_salt = self.hasher.generate_salt()?;
        let hashed_secret = self.hasher.hash(&secret, &secret_salt)?;
        let password_salt = self.hasher.generate_salt()?;
        let hashed_password = self.hasher.hash(&req.password, &password_salt)?;
        let id = self
            .repository
            .insert_user(CreateUser {
                phone: req.phone,
                email: req.email,
                password: Some(hashed_password),
                password_salt: Some(password_salt),
                secret: hashed_secret.clone(),
                secret_salt: secret_salt.clone(),
            })
            .await?;
        self.cacher
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

    pub async fn verify_secret(
        &mut self,
        req: VerifySecretRequest<ID>,
    ) -> Result<(), Box<dyn StdErr>> {
        if let Some(SecretPair {
            hashed_secret,
            secret_salt,
        }) = self
            .cacher
            .get_user_secret(req.id.clone())
            .await
            .unwrap_or_else(|err| {
                eprintln!("{:?}", err);
                None
            })
        {
            if self.hasher.hash(&req.secret, secret_salt)? == hashed_secret {
                return Ok(());
            }
            return Err(Box::new(Error::ServiceError(
                "用户不存在或凭证不正确".to_owned(),
            )));
        }
        if let Some(user) = self
            .repository
            .fetch_user(QueryUser {
                id_eq: Some(req.id),
                ..Default::default()
            })
            .await?
        {
            if self.hasher.hash(&req.secret, &user.secret_salt)? != user.secret {
                return Err(Box::new(Error::ServiceError(
                    "用户不存在或凭证不正确".to_owned(),
                )));
            }
            return Ok(());
        }
        Err(Box::new(Error::ServiceError(
            "用户不存在或凭证不正确".to_owned(),
        )))
    }

    pub async fn login(&mut self, req: LoginRequest) -> Result<LoginResponse<ID>, Box<dyn StdErr>> {
        if req.phone.is_none() && req.email.is_none() {
            return Err(Box::new(Error::ServiceError(
                "手机号与邮箱至少提供一个".to_owned(),
            )));
        }
        if let Some(user) = self
            .repository
            .fetch_user(QueryUser {
                phone_eq: req.phone,
                email_eq: req.email,
                ..Default::default()
            })
            .await?
        {
            if user.password.is_none() || user.password_salt.is_none() {
                return Err(Box::new(Error::ServiceError("不支持的登录方式".to_owned())));
            }
            if self
                .hasher
                .hash(&req.password, user.password_salt.unwrap())?
                != user.password.unwrap()
            {
                return Err(Box::new(Error::ServiceError(
                    "用户不存在或凭证不正确".to_owned(),
                )));
            }
            let secret = self.secret_generator.generate_secret()?;
            let salt = self.hasher.generate_salt()?;
            let hashed_secret = self.hasher.hash(&secret, &salt)?;
            let affected = self
                .repository
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
                return Err(Box::new(Error::ServiceError("更新用户密钥失败".to_owned())));
            }
            self.cacher
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
        Err(Box::new(Error::ServiceError(
            "用户不存在或凭证不正确".to_owned(),
        )))
    }
}
