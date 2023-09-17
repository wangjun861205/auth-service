#![feature(async_fn_in_trait)]

pub mod core;
pub mod handlers;
pub mod impls;

use core::service::Service;
use std::convert::Infallible;
use std::env;
use std::{error::Error as StdErr, fmt::Display};
use tokio::sync::Mutex;

use crate::{
    handlers::{SecretHeader, UIDHeader},
    impls::{
        cachers::redis::RedisCacher, hashers::sha::ShaHasher,
        repositories::postgresql::PostgresqlRepository,
        secret_generators::random::RandomSecretGenerator,
        verify_code_managers::fake::FakeVerifyCodeManager,
    },
};
use actix_web::{
    middleware::Logger,
    web::{post, put, scope, Data},
    App, HttpServer,
};
use core::{
    cacher::Cacher, hasher::Hasher, repository::Repository, verify_code_manager::VerifyCodeManager,
};
use handlers::{login, register_user, send_verify_code, verify_secret};
use serde::Serialize;
use sqlx::PgPool;

pub trait RepositoryFactory<R, ID>
where
    R: Repository<ID> + 'static,
    ID: Default + Clone + Serialize + Display,
{
    async fn new_repository(&self) -> Result<R, Box<dyn StdErr>>;
}

pub trait VerifyCodeManagerFactory<V>
where
    V: VerifyCodeManager + 'static,
{
    async fn new_verify_code_manager(&self) -> Result<V, Box<dyn StdErr>>;
}

pub trait HasherFactory<H>
where
    H: Hasher + 'static,
{
    async fn new_hasher(&self) -> Result<H, Box<dyn StdErr>>;
}

pub trait CacherFactory<C, ID>
where
    C: Cacher<ID> + 'static,
    ID: Default + Clone + Serialize + Display,
{
    async fn new_cacher(&self) -> Result<C, Box<dyn StdErr>>;
}

#[actix_web::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init_from_env(
        env_logger::Env::new()
            .default_filter_or(dotenv::var("LOG_LEVEL").unwrap_or("info".to_string())),
    );
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pg_pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");
    let users_redis_url = env::var("USERS_REDIS_URL").expect("REDIS_URL must be set");
    let users_client =
        redis::Client::open(users_redis_url).expect("Failed to connect to users redis database");

    let service = Service::new(
        PostgresqlRepository::new(pg_pool.clone()),
        RedisCacher::<i32>::new(users_client),
        ShaHasher {},
        RandomSecretGenerator {},
        FakeVerifyCodeManager::new(),
    );
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new(
                &env::var("LOG_FORMAT").unwrap_or("%{User-Agent}i\n%s\n%a\n%r\n%T".to_owned()),
            ))
            .app_data(Data::new(Mutex::new(service.clone())))
            .app_data(Data::new(UIDHeader(
                env::var("UID_HEADER").unwrap_or("X-UID".to_owned()),
            )))
            .app_data(Data::new(SecretHeader(
                env::var("SECRET_HEADER").unwrap_or("X-SECRET".to_owned()),
            )))
            .service(
                scope("")
                    .route(
                        "users",
                        post().to(register_user::<
                            PostgresqlRepository,
                            RandomSecretGenerator,
                            FakeVerifyCodeManager,
                            ShaHasher,
                            RedisCacher<i32>,
                            i32,
                        >),
                    )
                    .route(
                        "login",
                        put().to(login::<
                            PostgresqlRepository,
                            RandomSecretGenerator,
                            ShaHasher,
                            RedisCacher<i32>,
                            FakeVerifyCodeManager,
                            i32,
                        >),
                    )
                    .route(
                        "verify_secret",
                        put().to(verify_secret::<
                            PostgresqlRepository,
                            ShaHasher,
                            RedisCacher<i32>,
                            RandomSecretGenerator,
                            FakeVerifyCodeManager,
                            i32,
                            _,
                        >),
                    )
                    .route(
                        "send_verify_code",
                        put().to(send_verify_code::<
                            PostgresqlRepository,
                            RedisCacher<i32>,
                            ShaHasher,
                            RandomSecretGenerator,
                            FakeVerifyCodeManager,
                            i32,
                        >),
                    ),
            )
    })
    .bind("localhost:8000")
    .unwrap()
    .run()
    .await
    .unwrap()
}
