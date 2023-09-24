#![feature(async_fn_in_trait)]

pub mod core;
pub mod handlers;
pub mod impls;

use std::env;
use tokio::sync::Mutex;

use crate::{
    core::service::Service,
    handlers::{login, register_user, verify_secret, SecretHeader, UIDHeader},
    impls::{
        cachers::redis::RedisCacher, hashers::sha::ShaHasher,
        repositories::postgresql::PostgresqlRepository,
        secret_generators::random::RandomSecretGenerator,
    },
};
use actix_web::{
    middleware::Logger,
    web::{post, put, scope, Data},
    App, HttpServer,
};
use nb_from_env::{FromEnv, FromEnvDerive};
use sqlx::PgPool;

#[derive(FromEnvDerive)]
struct ServerConfig {
    database_url: String,
    redis_url: String,
    server_address: String,
    log_level: Option<String>,
    log_format: Option<String>,
    uid_header: Option<String>,
    secret_header: Option<String>,
}

pub async fn start_default_server() {
    dotenv::dotenv().ok();
    let config = ServerConfig::from_env();
    env_logger::init_from_env(
        env_logger::Env::new().default_filter_or(config.log_level.unwrap_or("info".to_owned())),
    );
    let pg_pool = PgPool::connect(&config.database_url)
        .await
        .expect("Failed to connect to Postgres");
    let users_client =
        redis::Client::open(config.redis_url).expect("Failed to connect to users redis database");

    let service = Service::new(
        PostgresqlRepository::new(pg_pool.clone()),
        RedisCacher::<i32>::new(users_client),
        ShaHasher {},
        RandomSecretGenerator {},
    );
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new(
                &config
                    .log_format
                    .clone()
                    .unwrap_or("%{User-Agent}i\n%s\n%a\n%r\n%T".to_owned()),
            ))
            .app_data(Data::new(Mutex::new(service.clone())))
            .app_data(Data::new(UIDHeader(
                config.uid_header.clone().unwrap_or("X-UID".to_owned()),
            )))
            .app_data(Data::new(SecretHeader(
                config
                    .secret_header
                    .clone()
                    .unwrap_or("X-SECRET".to_owned()),
            )))
            .service(
                scope("")
                    .route(
                        "users",
                        post().to(register_user::<
                            PostgresqlRepository,
                            RandomSecretGenerator,
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
                            i32,
                            _,
                        >),
                    ),
            )
    })
    .bind(config.server_address)
    .unwrap()
    .run()
    .await
    .unwrap()
}
