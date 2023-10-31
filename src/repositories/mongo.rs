use anyhow::Error;
use mongodb::{
    bson::{doc, Document},
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
            .find_one(doc! {"phone": phone}, None)
            .await?)
    }

    async fn insert_user(&self, user: &CreateUser) -> Result<String, Error> {
        let res = self
            .db
            .collection::<CreateUser>("users")
            .insert_one(user, None)
            .await?;
        Ok(res
            .inserted_id
            .as_object_id()
            .ok_or(Error::msg("未找到object id"))?
            .to_hex())
    }
}
