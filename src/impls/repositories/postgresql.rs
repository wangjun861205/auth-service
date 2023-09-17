use sqlx::{postgres::PgPool, query_as, query_scalar, Decode, Encode, FromRow, Postgres, Type};

use crate::core::{
    entities::{CreateUser, QueryUser, UpdateUser},
    repository::Repository,
};
use chrono::{DateTime, Local};
use std::{
    error::Error as StdErr,
    fmt::{Debug, Display},
};

#[derive(Debug, Clone)]
pub struct PostgresqlRepository {
    pool: PgPool,
}

impl PostgresqlRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
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
pub struct User<ID>
where
    ID: Unpin,
{
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

impl<ID> Repository<ID> for PostgresqlRepository
where
    for<'i> ID: Default
        + Clone
        + Display
        + Send
        + Encode<'i, Postgres>
        + Decode<'i, Postgres>
        + Type<Postgres>
        + Unpin,
{
    async fn fetch_user(
        &mut self,
        query: QueryUser<ID>,
    ) -> Result<Option<crate::core::entities::User<ID>>, Box<dyn StdErr>> {
        if let Some(user) = query_as::<Postgres, User<ID>>(
            "SELECT * 
        FROM users 
        WHERE ($1 IS NULL OR id = $1)
        AND ($2 IS NULL OR phone = $2)
        AND ($3 IS NULL OR email = $3)",
        )
        .bind(query.id_eq)
        .bind(query.phone_eq)
        .bind(query.email_eq)
        .fetch_optional(&self.pool)
        .await?
        {
            return Ok(Some(crate::core::entities::User {
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

    async fn insert_user(&self, user: CreateUser) -> Result<ID, Box<dyn StdErr>> {
        Ok(query_scalar(
            "INSERT INTO users (
            phone, 
            email, 
            password_salt, 
            password, 
            secret, 
            secret_salt, 
            ) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
        )
        .bind(user.phone)
        .bind(user.email)
        .bind(user.password_salt)
        .bind(user.password)
        .bind(user.secret)
        .bind(user.secret_salt)
        .fetch_one(&self.pool)
        .await?)
    }
    async fn update_user(
        &self,
        query: QueryUser<ID>,
        user: UpdateUser,
    ) -> Result<i64, Box<dyn StdErr>> {
        Ok(query_scalar(
            "WITH u AS (
                UPDATE users SET
                secret = COALESCE($1, secret),
                secret_salt = COALESCE($2, secret_salt)
                WHERE ($3 IS NULL OR id = $3)
                AND ($4 IS NULL OR phone = $4)
                AND ($5 IS NULL OR email = $5)
                RETURNING *
            )
            SELECT count(*) FROM u",
        )
        .bind(user.secret)
        .bind(user.secret_salt)
        .bind(query.id_eq)
        .bind(query.phone_eq)
        .bind(query.email_eq)
        .fetch_one(&self.pool)
        .await?)
    }
}
