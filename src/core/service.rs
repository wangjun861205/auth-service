use anyhow::Error;

use crate::core::{
    entities::{CreateUser, QueryUser, UpdateUser},
    hasher::Hasher,
    repository::Repository,
    token_manager::TokenManager,
};

#[derive(Debug, Clone)]
pub struct Service<R, H, T>
where
    R: Repository + Clone,
    H: Hasher + Clone,
    T: TokenManager + Clone,
{
    pub repository: R,
    pub hasher: H,
    pub token_manager: T,
}

pub struct RegisterUserRequest {
    pub phone: String,
    pub password: String,
    pub verify_code: String,
}

pub struct RegisterUserResponse {
    pub id: String,
    pub token: String,
}

pub struct LoginRequest {
    pub phone: Option<String>,
    pub password: String,
}

pub struct LoginResponse {
    pub token: String,
}

impl<R, H, T> Service<R, H, T>
where
    R: Repository + Clone,
    H: Hasher + Clone,
    T: TokenManager + Clone,
{
    pub fn new(repository: R, hasher: H, token_manager: T) -> Self {
        Self {
            repository,
            hasher,
            token_manager,
        }
    }

    pub async fn register_user(&mut self, req: RegisterUserRequest) -> Result<String, Error> {
        if self.repository.exists_user(&req.phone).await? {
            return Err(Error::msg("手机号已被注册"));
        }

        let password_salt = self.hasher.generate_salt()?;
        let hashed_password = self.hasher.hash(&req.password, &password_salt)?;
        Ok(self
            .repository
            .insert_user(&CreateUser {
                phone: req.phone,
                email: req.email,
                password: Some(hashed_password),
                password_salt: Some(password_salt),
            })
            .await?)
    }

    pub async fn verify_token(&mut self, token: impl AsRef<String>) -> Result<String, Error> {
        self.token_manager.verify_token(token).await
    }

    pub async fn login(&mut self, req: LoginRequest) -> Result<LoginResponse, Error> {
        if let Some(user) = self.repository.fetch_user(&req.phone).await? {
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
