use chrono::Utc;
use mongodb::{
    bson::{doc, Document},
    options::FindOneOptions,
    Database,
};

use crate::core::{
    entities::{CreateUser, User},
    error::Error,
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
            .await
            .map_err(|e| Error::FailedToCheckExists(Box::new(e)))?
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
                        "id": { "$toString": "$_id" },
                        "phone": 1,
                        "password": 1,
                        "password_salt": 1,
                        "created_at": 1,
                        "updated_at": 1,
                    })
                    .build(),
            )
            .await
            .map_err(|e| Error::FailedToFetchUser(Box::new(e)))?)
    }

    async fn insert_user(&self, user: &CreateUser) -> Result<String, Error> {
        let res = self
            .db
            .collection::<Document>("users")
            .insert_one(
                doc! {
                    "phone": &user.identifier,
                    "password": &user.password,
                    "password_salt": &user.password_salt,
                    "created_at": Utc::now().to_rfc3339(),
                    "updated_at": Utc::now().to_rfc3339(),
                },
                None,
            )
            .await
            .map_err(|e| Error::FailedToInsertUser(Box::new(e)))?;
        Ok(res
            .inserted_id
            .as_object_id()
            .ok_or(Error::FailedToInsertUser(Box::new(
                "未找到object id".to_owned(),
            )))?
            .to_hex())
    }
}
