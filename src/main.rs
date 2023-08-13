#![feature(async_fn_in_trait)]

use std::collections::HashMap;
use std::env;
use std::fmt::Display;
use std::sync::{Arc, Mutex};

use crate::hasher::sha::ShaHasher;
use actix_web::{
    web::{post, put, scope, Data},
    App, HttpServer,
};
use cacher::redis::{RedisCacher, RedisCacherFactory};
use handlers::{login, register_app, register_user, send_verify_code, verify_secret};
use hasher::sha::ShaHasherFactory;
use repositories::postgresql::{PostgresqlRepository, PostgresqlRepositoryFactory};
use secret_generators::random::RandomSecretGenerator;
use serde::Serialize;
use services::{Cacher, Hasher, Repository, VerifyCodeManager};
use sqlx::PgPool;
use verify_code_managers::fake::{FakeVerifyCodeManager, FakeVerifyCodeManagerFactory};

pub(crate) mod cacher;
pub(crate) mod error;
pub(crate) mod handlers;
pub(crate) mod hasher;
pub(crate) mod models;
pub(crate) mod repositories;
pub(crate) mod secret_generators;
pub(crate) mod services;
pub(crate) mod verify_code_managers;

pub trait RepositoryFactory<R, ID>
where
    R: Repository<ID> + 'static,
    ID: Default + Clone + Serialize + Display,
{
    async fn new_repository(&self) -> Result<R, error::Error>;
}

pub trait VerifyCodeManagerFactory<V>
where
    V: VerifyCodeManager + 'static,
{
    async fn new_verify_code_manager(&self) -> Result<V, error::Error>;
}

pub trait HasherFactory<H>
where
    H: Hasher + 'static,
{
    async fn new_hasher(&self) -> Result<H, error::Error>;
}

pub trait CacherFactory<C, ID>
where
    C: Cacher<ID> + 'static,
    ID: Default + Clone + Serialize + Display,
{
    async fn new_cacher(&self) -> Result<C, error::Error>;
}

#[actix_web::main]
async fn main() {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pg_pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");
    let (phone_map, email_map) = (
        Arc::new(Mutex::new(HashMap::new())),
        Arc::new(Mutex::new(HashMap::new())),
    );
    let apps_redis_url = env::var("APPS_REDIS_URL").expect("REDIS_URL must be set");
    let users_redis_url = env::var("USERS_REDIS_URL").expect("REDIS_URL must be set");
    let apps_client =
        redis::Client::open(apps_redis_url).expect("Failed to connect to apps redis database");
    let users_client =
        redis::Client::open(users_redis_url).expect("Failed to connect to users redis database");
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(PostgresqlRepositoryFactory::new(pg_pool.clone())))
            .app_data(Data::new(FakeVerifyCodeManagerFactory::new(
                email_map.clone(),
                phone_map.clone(),
            )))
            .app_data(Data::new(ShaHasherFactory::new()))
            .app_data(Data::new(RedisCacherFactory::new(
                apps_client.clone(),
                users_client.clone(),
            )))
            .service(scope("/api/v1/apps").route(
                "",
                post().to(register_app::<
                    PostgresqlRepository,
                    RandomSecretGenerator,
                    ShaHasher,
                    RedisCacher<String>,
                    String,
                >),
            ))
            .service(
                scope("/api/v1/users")
                    .route(
                        "",
                        post().to(register_user::<
                            PostgresqlRepository,
                            RandomSecretGenerator,
                            FakeVerifyCodeManager,
                            ShaHasher,
                            RedisCacher<String>,
                            String,
                        >),
                    )
                    .route(
                        "login",
                        put().to(login::<
                            PostgresqlRepository,
                            RandomSecretGenerator,
                            ShaHasher,
                            RedisCacher<String>,
                            String,
                        >),
                    )
                    .route(
                        "verify_secret",
                        put().to(verify_secret::<
                            PostgresqlRepository,
                            ShaHasher,
                            RedisCacher<String>,
                            String,
                        >),
                    )
                    .route(
                        "send_verify_code",
                        put().to(send_verify_code::<FakeVerifyCodeManager>),
                    ),
            )
    })
    .bind("localhost:8000")
    .unwrap()
    .run()
    .await
    .unwrap()
}
