use crate::error::Error;
use crate::services::{self, Repository, SecretGenerator, VerifyCodeManager};
use actix_web::{web::Json, FromRequest};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterAppRequest {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterAppResponse<ID>
where
    ID: Default + Clone,
{
    id: ID,
    secret: String,
}

pub async fn register_app<'a, R, S, ID>(
    mut repository: R,
    secret_generator: S,
    Json(req): Json<RegisterAppRequest>,
) -> Result<Json<RegisterAppResponse<ID>>, Error>
where
    R: Repository<ID> + FromRequest,
    S: SecretGenerator + FromRequest,
    ID: Default + Clone + Serialize,
{
    let res = services::register_app(
        &mut repository,
        &secret_generator,
        services::RegisterAppRequest { name: req.name },
    )
    .await?;
    Ok(Json(RegisterAppResponse {
        id: res.id,
        secret: res.secret,
    }))
}
