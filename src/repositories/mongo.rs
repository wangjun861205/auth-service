use anyhow::Error;
use chrono::Utc;
use mongodb::{
    bson::{doc, Document},
    options::FindOneOptions,
    Database,
};

use crate::core::{
    entities::{CreateUser, User},
    repository::Repository,
};

#[derive(Debug, Clone)]
pub struct MongodbRepository {
    db: Database,
}

impl MongodbRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

impl Repository for MongodbRepository {
    async fn exists_user(&self, phone: &str) -> Result<bool, Error> {
        Ok(self
            .db
            .collection::<Document>("users")
            .count_documents(doc! {"phone": phone}, None)
            .await?
            == 1)
    }
    async fn fetch_user(&self, phone: &str) -> Result<Option<User>, Error> {
        Ok(self
            .db
            .collection::<User>("users")
            .find_one(
                doc! {"phone": phone},
                FindOneOptions::builder()
                    .projection(doc! {
                        "id": "$id",
                        "phone": 1,
                        "password": 1,
                        "password_salk": 1,
                        "created_ad": 1,
                        "updated_at": 1,
                    })
                    .build(),
            )
            .await?)
    }

    async fn insert_user(&self, user: &CreateUser) -> Result<String, Error> {
        let res = self
            .db
            .collection::<Document>("users")
            .insert_one(
                doc! {
                    "phone": &user.phone,
                    "password": &user.password,
                    "password_salt": &user.password_salt,
                    "created_at": Utc::now().to_rfc3339(),
                    "updated_at": Utc::now().to_rfc3339(),
                },
                None,
            )
            .await?;
        Ok(res
            .inserted_id
            .as_object_id()
            .ok_or(Error::msg("未找到object id"))?
            .to_hex())
    }
}
