use actix_web::{web::Data, FromRequest};
use sqlx::{postgres::PgPool, query_as, query_scalar, FromRow, Postgres, Transaction};

use crate::{error::Error, models::QueryApp, services::Repository, RepositoryFactory};
use chrono::{DateTime, Local};
use std::{future::Future, pin::Pin};

pub struct PostgresqlRepositoryFactory {
    pool: PgPool,
}

impl PostgresqlRepositoryFactory {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl RepositoryFactory<PostgresqlRepository, String> for PostgresqlRepositoryFactory {
    async fn new_repository(&self) -> Result<PostgresqlRepository, Error> {
        let tx = self.pool.begin().await?;
        Ok(PostgresqlRepository { tx })
    }
}

pub struct PostgresqlRepository {
    tx: Transaction<'static, Postgres>,
}

#[derive(Debug, FromRow)]
pub struct App<String> {
    pub id: String,
    pub name: String,
    pub secret: String,
    pub secret_salt: String,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(Debug, FromRow)]
pub struct User<ID> {
    pub id: ID,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub password_salt: Option<String>,
    pub password: Option<String>,
    pub secret: String,
    pub secret_salt: String,
    pub app_id: ID,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

impl Repository<String> for PostgresqlRepository {
    async fn fetch_app(
        &mut self,
        query: QueryApp<String>,
    ) -> Result<Option<crate::models::App<String>>, Error> {
        if let Some(app) =
            query_as::<Postgres, App<String>>("SELECT * FROM apps WHERE $1 IS NULL OR id = $1")
                .bind(query.id_eq)
                .fetch_optional(&mut *self.tx)
                .await?
        {
            return Ok(Some(crate::models::App {
                id: app.id,
                name: app.name,
                secret: app.secret,
                secret_salt: app.secret_salt,
                created_at: app.created_at,
                updated_at: app.updated_at,
            }));
        }
        Ok(None)
    }

    async fn exists_app(&mut self, query: QueryApp<String>) -> Result<bool, Error> {
        Ok(query_scalar(
            "SELECT EXISTS(
            SELECT 1 FROM apps 
            WHERE $1 IS NULL OR id = $1
        )",
        )
        .bind(query.id_eq)
        .fetch_one(&mut *self.tx)
        .await?)
    }
    async fn fetch_user(
        &mut self,
        query: crate::models::QueryUser<String>,
    ) -> Result<Option<crate::models::User<String>>, Error> {
        if let Some(user) = query_as::<Postgres, User<String>>(
            "SELECT * 
        FROM users 
        WHERE ($1 IS NULL OR id = $1)
        AND ($2 IS NULL OR phone = $2)
        AND ($3 IS NULL OR email = $3)
        AND ($4 IS NULL OR app_id = $4)",
        )
        .bind(query.id_eq)
        .bind(query.phone_eq)
        .bind(query.email_eq)
        .bind(query.app_id_eq)
        .fetch_optional(&mut *self.tx)
        .await?
        {
            return Ok(Some(crate::models::User {
                id: user.id,
                phone: user.phone,
                email: user.email,
                password_salt: user.password_salt,
                password: user.password,
                secret: user.secret,
                secret_salt: user.secret_salt,
                app_id: user.app_id,
                created_at: user.created_at,
                updated_at: user.updated_at,
            }));
        }
        Ok(None)
    }
    async fn insert_app(&mut self, app: crate::models::CreateApp) -> Result<String, Error> {
        Ok(query_scalar(
            "INSERT INTO apps (name, secret, secret_salt) 
            VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(app.name)
        .bind(app.secret)
        .bind(app.secret_salt)
        .fetch_one(&mut *self.tx)
        .await?)
    }
    async fn insert_user(
        &mut self,
        user: crate::models::CreateUser<String>,
    ) -> Result<String, Error> {
        Ok(query_scalar(
            "INSERT INTO users (
            phone, 
            email, 
            password_salt, 
            password, 
            secret, 
            secret_salt, 
            app_id) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id",
        )
        .bind(user.phone)
        .bind(user.email)
        .bind(user.password_salt)
        .bind(user.password)
        .bind(user.secret)
        .bind(user.secret_salt)
        .bind(user.app_id)
        .fetch_one(&mut *self.tx)
        .await?)
    }
    async fn update_user(
        &mut self,
        query: crate::models::QueryUser<String>,
        user: crate::models::UpdateUser,
    ) -> Result<i64, Error> {
        Ok(query_scalar(
            "WITH u AS (
                UPDATE users SET
                secret = COALESCE($1, secret),
                secret_salt = COALESCE($2, secret_salt)
                WHERE ($3 IS NULL OR id = $3)
                AND ($4 IS NULL OR phone = $4)
                AND ($5 IS NULL OR email = $5)
                AND ($6 IS NULL OR app_id = $6)
                RETURNING *
            )
            SELECT count(*) FROM u",
        )
        .bind(user.secret)
        .bind(user.secret_salt)
        .bind(query.id_eq)
        .bind(query.phone_eq)
        .bind(query.email_eq)
        .bind(query.app_id_eq)
        .fetch_one(&mut *self.tx)
        .await?)
    }

    async fn commit(self) -> Result<(), Error> {
        Ok(self.tx.commit().await?)
    }
}

impl FromRequest for PostgresqlRepository {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;
    fn extract(req: &actix_web::HttpRequest) -> Self::Future {
        if let Some(factory) = req.app_data::<Data<PostgresqlRepositoryFactory>>() {
            let factory = factory.clone();
            return Box::pin(async move {
                let repo = factory.new_repository().await?;
                Ok(repo)
            });
        }
        Box::pin(async move { Err(Error::RepositoryError("无工厂函数".into())) })
    }

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        if let Some(factory) = req.app_data::<Data<PostgresqlRepositoryFactory>>() {
            let factory = factory.clone();
            return Box::pin(async move {
                let repo = factory.new_repository().await?;
                Ok(repo)
            });
        }
        Box::pin(async move { Err(Error::RepositoryError("无工厂函数".into())) })
    }
}
