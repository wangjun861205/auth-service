#![feature(async_fn_in_trait)]

use actix_web::{
    body::MessageBody,
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    web::{post, Data},
    App, Error, FromRequest, HttpServer,
};
use handlers::register_app;
use repositories::postgresql::{PostgresqlRepository, PostgresqlRepositoryFactory};
use secret_generators::random::RandomSecretGenerator;
use serde::Serialize;
use services::{Repository, SecretGenerator, VerifyCodeManager};
use sqlx::PgPool;
use verify_code_managers::fake::FakeVerifyCodeManager;

pub(crate) mod error;
pub(crate) mod handlers;
pub(crate) mod models;
pub(crate) mod repositories;
pub(crate) mod secret_generators;
pub(crate) mod services;
pub(crate) mod verify_code_managers;

pub trait RepositoryFactory<R, ID>
where
    R: Repository<ID> + 'static,
    ID: Default + Clone + Serialize,
{
    async fn new_repository(&self) -> Result<R, error::Error>;
}

pub fn new_app<RF, R, V, S, ID>(
    repository_factory: RF,
) -> App<
    impl ServiceFactory<
        ServiceRequest,
        Config = (),
        Response = ServiceResponse<impl MessageBody>,
        Error = Error,
        InitError = (),
    >,
>
where
    RF: RepositoryFactory<R, ID> + 'static,
    R: Repository<ID> + FromRequest + 'static,
    V: VerifyCodeManager + FromRequest + 'static,
    S: SecretGenerator + FromRequest + 'static,
    ID: Default + Clone + Serialize + 'static,
{
    App::new()
        .app_data(Data::new(repository_factory))
        .route("/api/v1/register_app", post().to(register_app::<R, S, ID>))
}

#[actix_web::main]
async fn main() {
    let pg_pool = PgPool::connect("postgres://postgres:postgres@localhost:5432/nbauth")
        .await
        .expect("Failed to connect to Postgres");
    HttpServer::new(move || {
        new_app::<
            PostgresqlRepositoryFactory,
            PostgresqlRepository,
            FakeVerifyCodeManager,
            RandomSecretGenerator,
            String,
        >(PostgresqlRepositoryFactory::new(pg_pool.clone()))
    })
    .bind("localhost:8000")
    .unwrap()
    .run()
    .await
    .unwrap()
}
